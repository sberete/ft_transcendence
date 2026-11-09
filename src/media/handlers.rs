// HTTP handlers for the media catalogue: searching across both
// providers in parallel, and fetching one item's playable details. The
// details endpoint also doubles as the single instrumentation point
// for watch history (see `log_watch`).

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;

use super::{archive, peertube, MediaClient, MediaDetails, MediaItem, ProviderId};
use crate::auth::AuthUser;
use crate::errors::ApiError;

const MAX_QUERY_LEN: usize = 100;

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
    // Optional: restricts the search to a single source (the frontend's themed rows).
    provider: Option<ProviderId>,
}

// Every route requires being logged in: otherwise anyone could use our
// server as a free relay to the underlying sources.
//
// Queries both providers in parallel. A source that's down doesn't fail
// the whole search: it's logged and the other results are kept
// (equivalent to Movix's Promise.allSettled).
pub async fn search(_auth: AuthUser, Extension(client): Extension<MediaClient>, Query(params): Query<SearchParams>)
    -> Result<Json<Vec<MediaItem>>, ApiError>
{
    let query = params.q.trim();
    if query.is_empty() || query.len() > MAX_QUERY_LEN {
        return Err(ApiError::BadRequest(format!(
            "q must be between 1 and {MAX_QUERY_LEN} characters"
        )));
    }

    let wants = |provider| params.provider.is_none_or(|only| only == provider);

    let (archive_res, peertube_res) = tokio::join!(
        async {
            if wants(ProviderId::Archive) { archive::search(&client, query).await } else { Ok(Vec::new()) }
        },
        async {
            if wants(ProviderId::Peertube) { peertube::search(&client, query).await } else { Ok(Vec::new()) }
        },
    );

    let mut items = Vec::new();
    for (provider, result) in [(ProviderId::Archive, archive_res), (ProviderId::Peertube, peertube_res)] {
        match result {
            Ok(found) => items.extend(found),
            Err(err) => log::warn!("media search failed for {provider:?}: {err}"),
        }
    }

    Ok(Json(items))
}

// Fetches one item's full details (description, backdrop, playable
// streams) after validating the id's format, and logs it as watched.
pub async fn details(
    auth: AuthUser,
    Extension(client): Extension<MediaClient>,
    State(pool): State<PgPool>,
    Path((provider, id)): Path<(ProviderId, String)>,
)
    -> Result<Json<MediaDetails>, ApiError>
{
    let details = match provider {
        ProviderId::Archive => {
            if !archive::is_valid_id(&id) {
                return Err(ApiError::BadRequest("invalid archive id".into()));
            }
            archive::details(&client, &id).await
        }
        ProviderId::Peertube => {
            if !peertube::is_valid_id(&id) {
                return Err(ApiError::BadRequest("invalid peertube id".into()));
            }
            peertube::details(&client, &id).await
        }
    }
    .map_err(|err| {
        log::warn!("media details failed for {provider:?}/{id}: {err}");
        ApiError::BadGateway("the video source is unavailable, try again later".into())
    })?;

    if details.streams.is_empty() {
        return Err(ApiError::NotFound("no playable stream for this media".into()));
    }

    // The one instrumented spot for the recommendation system: both
    // WatchView AND RoomView go through this same endpoint (RoomView calls
    // it again whenever the room's video changes), so a single line covers
    // both solo and in-room watching. Best-effort, must never fail the
    // video display itself.
    log_watch(&pool, auth.user_id, provider, &id, &details.item).await;

    Ok(Json(details))
}

// Records (or bumps the timestamp of) one watched item, best-effort.
async fn log_watch(pool: &PgPool, user_id: i64, provider: ProviderId, id: &str, item: &MediaItem) {
    if let Err(err) = sqlx::query(
        "INSERT INTO watch_history (user_id, provider, media_id, title, year, thumbnail_url, watched_at)
         VALUES ($1, $2, $3, $4, $5, $6, now())
         ON CONFLICT (user_id, provider, media_id) DO UPDATE SET watched_at = now()",
    )
    .bind(user_id)
    .bind(provider.as_str())
    .bind(id)
    .bind(&item.title)
    .bind(item.year)
    .bind(&item.thumbnail)
    .execute(pool)
    .await
    {
        log::warn!("failed to log watch_history for user {user_id}: {err}");
    }
}
