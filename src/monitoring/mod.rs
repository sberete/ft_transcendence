// Prometheus metrics for the backend: HTTP request counters, durations
// and in-flight requests, exposed as text on a separate internal port
// (METRICS_PORT, 9091 by default). That port is never proxied by nginx,
// which only forwards to the main API port: the metrics stay reachable
// from the Compose network (Prometheus) but never from the browser.
//
// Named `monitoring` rather than `metrics` to avoid clashing with the
// `metrics` crate used below.

use std::time::{Duration, Instant};

use axum::extract::{MatchedPath, Request};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram, Unit};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

const REQUESTS_TOTAL: &str = "http_requests_total";
const REQUEST_DURATION: &str = "http_request_duration_seconds";
const REQUESTS_IN_FLIGHT: &str = "http_requests_in_flight";
const ACTIVE_ROOMS: &str = "syncwatch_active_rooms";
const ROOM_MEMBERS: &str = "syncwatch_room_members";

// Histogram buckets (in seconds) for request durations: fine-grained
// under 100 ms where most API calls land, coarser above. Real buckets
// (instead of the exporter's default client-side quantiles) let Grafana
// compute the 95th percentile with histogram_quantile().
const DURATION_BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

// Installs the global metrics recorder. Must be called once, before any
// request is handled; the returned handle renders the metrics as text.
pub fn install()
    -> PrometheusHandle
{
    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(Matcher::Full(REQUEST_DURATION.to_string()), DURATION_BUCKETS)
        .expect("invalid histogram buckets")
        .install_recorder()
        .expect("failed to install the Prometheus recorder");

    describe_counter!(REQUESTS_TOTAL, "HTTP requests handled, by method, route and status code.");
    describe_histogram!(REQUEST_DURATION, Unit::Seconds, "HTTP request duration, by method, route and status code.");
    describe_gauge!(REQUESTS_IN_FLIGHT, "HTTP requests currently being handled.");
    describe_gauge!(ACTIVE_ROOMS, "Watch rooms with at least one connected member.");
    describe_gauge!(ROOM_MEMBERS, "People connected to a watch room, all rooms combined.");

    // Published at 0 right away, so the dashboard shows "0" rather than
    // "No data" until the first room is opened.
    set_live_rooms(0, 0);

    // Without the exporter's built-in HTTP listener, nobody drains the
    // histograms' internal buffers: do it periodically so memory stays flat.
    let upkeep = handle.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            upkeep.run_upkeep();
        }
    });

    handle
}

// Starts the internal /metrics listener in the background. Binding
// happens here (not in the spawned task) so a port conflict stops the
// backend at startup instead of silently leaving it without metrics.
pub async fn serve(handle: PrometheusHandle, port: u16)
    -> std::io::Result<()>
{
    let app = Router::new().route("/metrics", get(move || std::future::ready(handle.render())));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    log::info!("Serving metrics on 0.0.0.0:{port}/metrics");
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            log::error!("metrics listener stopped: {err}");
        }
    });
    Ok(())
}

// Live room counters, updated by the WebSocket room registry on every
// join and leave (see ws::registry).
pub fn set_live_rooms(rooms: usize, members: usize)
{
    gauge!(ACTIVE_ROOMS).set(rooms as f64);
    gauge!(ROOM_MEMBERS).set(members as f64);
}

// Middleware recording every request that matched a route. Applied with
// `route_layer`, so unknown URLs (404 from the fallback) are not counted:
// their raw paths would create one time series per URL tried.
pub async fn track(req: Request, next: Next)
    -> Response
{
    let start = Instant::now();
    let method = method_label(req.method());
    let route = route_label(&req);

    let in_flight = gauge!(REQUESTS_IN_FLIGHT);
    in_flight.increment(1.0);
    let response = next.run(req).await;
    in_flight.decrement(1.0);

    let labels = [
        ("method", method),
        ("route", route),
        ("status", response.status().as_u16().to_string()),
    ];
    counter!(REQUESTS_TOTAL, &labels).increment(1);
    histogram!(REQUEST_DURATION, &labels).record(start.elapsed().as_secs_f64());

    response
}

// The route *pattern* (e.g. /rooms/{room_id}), never the actual URL: a
// label per room id or user id would make the number of series explode
// and leak identifiers into the metrics.
fn route_label(req: &Request)
    -> String
{
    // Avatars are served by ServeDir through nest_service, which gives
    // no MatchedPath: group every file under the fixed prefix.
    if req.uri().path().starts_with("/uploads/") {
        return "/uploads".to_string();
    }
    match req.extensions().get::<MatchedPath>() {
        Some(path) => path.as_str().to_string(),
        None => "unmatched".to_string(),
    }
}

// Standard methods only: a client can send any string as a method, which
// would otherwise create a new series each time.
fn method_label(method: &Method)
    -> String
{
    match *method {
        Method::GET | Method::POST | Method::PUT | Method::PATCH
        | Method::DELETE | Method::HEAD | Method::OPTIONS => method.as_str().to_string(),
        _ => "OTHER".to_string(),
    }
}
