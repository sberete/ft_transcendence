use axum::extract::{Path, Query};
use axum::{Extension, Json};
use serde::Deserialize;

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
    _auth: AuthUser,
    Extension(client): Extension<MediaClient>,
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

    Ok(Json(details))
}
