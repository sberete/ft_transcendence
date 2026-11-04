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
    // Optionnel : limite la recherche a une seule source (rangees thematiques du front).
    provider: Option<ProviderId>,
}

// Toutes les routes exigent d'etre connecte : sinon n'importe qui pourrait
// utiliser notre serveur comme relais gratuit vers les sources.
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

    // Les sources sont interrogees en parallele. Une source en panne ne fait
    // pas echouer la recherche : on la logue et on garde les autres resultats
    // (equivalent du Promise.allSettled de Movix).
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

    // Seul endroit instrumente pour le systeme de recommandation : WatchView
    // ET RoomView passent tous les deux par ce endpoint (RoomView le rappelle
    // des que la video du salon change), donc une seule ligne couvre le
    // visionnage solo et en salon. Best-effort, ne doit jamais faire echouer
    // l'affichage de la video elle-meme.
    log_watch(&pool, auth.user_id, provider, &id, &details.item).await;

    Ok(Json(details))
}

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
