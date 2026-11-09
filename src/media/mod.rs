// Aggregator for legal video sources, on the same principle as Movix:
// nothing is hosted here, we query external sources and hand the
// frontend direct links (MP4 / HLS) that it plays itself.
//
// Difference from Movix: only sources that explicitly allow this, via
// their public API, with no protection bypass.

mod archive;
mod handlers;
mod peertube;

use std::time::Duration;

use axum::routing::get;
use axum::Router;
use reqwest::{RequestBuilder, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// archive.org sometimes takes more than 10s to answer: below that, too many false failures.
const HTTP_TIMEOUT: Duration = Duration::from_secs(12);
const RETRY_DELAY: Duration = Duration::from_millis(500);
const MAX_RESULTS_PER_PROVIDER: usize = 12;

// REST routes for searching the catalogue and fetching one item's details.
pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/search", get(handlers::search))
        .route("/{provider}/{id}", get(handlers::details))
}

// Same validation rule as GET /media/{provider}/{id}, exposed for other
// modules (ws: playback.set_media) that also receive a provider/id pair
// from the client and must validate it before use.
pub fn is_valid_media_ref(provider: ProviderId, id: &str)
    -> bool
{
    match provider {
        ProviderId::Archive => archive::is_valid_id(id),
        ProviderId::Peertube => peertube::is_valid_id(id),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Archive,
    Peertube,
}

impl ProviderId {
    // For TEXT columns (watch_history.provider): same shape as the JSON
    // form (rename_all = "lowercase") but explicit rather than going through serde.
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::Archive => "archive",
            ProviderId::Peertube => "peertube",
        }
    }
}

// A single search result, the same shape regardless of the source.
#[derive(Serialize)]
pub struct MediaItem {
    provider: ProviderId,
    id: String,
    title: String,
    year: Option<i32>,
    thumbnail: Option<String>,
    duration_seconds: Option<u64>,
}

// Full detail sheet for one video: what the playback page displays.
#[derive(Serialize)]
pub struct MediaDetails {
    #[serde(flatten)]
    item: MediaItem,
    // May contain HTML (Archive): the frontend must render it as plain text.
    description: Option<String>,
    // Larger image than `thumbnail`, used as the page's background.
    backdrop: Option<String>,
    streams: Vec<StreamLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    Mp4,
    Hls,
}

// A link the frontend's player can read directly.
#[derive(Serialize)]
pub struct StreamLink {
    kind: StreamKind,
    url: String,
    quality: Option<String>,
}

// Shared between handlers via Extension (reqwest::Client wraps an Arc
// internally, so cloning it is cheap and reuses the connection pool).
#[derive(Clone)]
pub struct MediaClient {
    http: reqwest::Client,
    peertube_base: Url,
}

impl MediaClient {
    // Builds the shared HTTP client used to talk to both providers.
    pub fn new(peertube_url: &str)
        -> Self
    {
        // Honest User-Agent: we identify ourselves instead of impersonating a browser.
        let http = reqwest::Client::builder()
            .user_agent(concat!("ft_transcendence/", env!("CARGO_PKG_VERSION")))
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("failed to build HTTP client");

        let peertube_base = Url::parse(peertube_url).expect("PEERTUBE_URL must be a valid URL");

        MediaClient { http, peertube_base }
    }

    // GET + JSON decoding, with ONE retry if the failure looks transient
    // (timeout, connection, 5xx): archive.org regularly returns isolated
    // 502s. Never more than one retry: we don't want to hammer a source
    // that's already struggling.
    async fn get_json<T: DeserializeOwned>(&self, request: RequestBuilder)
        -> Result<T, reqwest::Error>
    {
        let retry = request.try_clone();
        match send_json(request).await {
            Err(err) if is_transient(&err) => match retry {
                Some(retry) => {
                    log::info!("transient upstream error, retrying once: {err}");
                    tokio::time::sleep(RETRY_DELAY).await;
                    send_json(retry).await
                }
                None => Err(err),
            },
            result => result,
        }
    }
}

// Sends a request and decodes its JSON body, treating any non-2xx status as an error.
async fn send_json<T: DeserializeOwned>(request: RequestBuilder)
    -> Result<T, reqwest::Error>
{
    request.send().await?.error_for_status()?.json().await
}

// Whether a request error is worth retrying once (timeout, connection
// failure, or a 5xx from the upstream server).
fn is_transient(err: &reqwest::Error)
    -> bool
{
    err.is_timeout() || err.is_connect() || err.status().is_some_and(|s| s.is_server_error())
}
