// HTTP handler implementing the hybrid recommendation pipeline:
// collaborative filtering (co-watch overlap between accounts) blended
// with content-based filtering (TF-IDF title similarity, see
// `scoring`), with a catalogue fallback when collaborative signal is
// sparse, and honest cold-start/empty results rather than a disguised
// "most watched" fallback.

use std::collections::HashMap;

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;
use crate::media::ProviderId;

use super::scoring::TfIdfModel;

const MAX_RESULTS: usize = 12;
const MAX_HISTORY: i64 = 200;
// Weight of the collaborative term in the final score. The term cancels
// itself out (becomes 0) when there's no neighbor sharing tastes with the
// user: content then takes over with no explicit if/else switch between
// the two modes - that's what makes this "hybrid".
const COLLAB_WEIGHT: f64 = 0.6;
const CONTENT_WEIGHT: f64 = 1.0 - COLLAB_WEIGHT;
// Below this many candidates found through collaborative filtering, we
// top up with the general catalogue (everything anyone has ever
// watched) to guarantee there's always something to propose beyond
// already-watched content, even for an account with no "neighbors".
const MIN_CANDIDATES_BEFORE_CATALOG_FALLBACK: usize = MAX_RESULTS * 2;

#[derive(sqlx::FromRow, Clone)]
struct WatchedRow {
    provider: String,
    media_id: String,
    title: String,
    year: Option<i32>,
    thumbnail_url: Option<String>,
}

type MediaKey = (String, String);

// Composite (provider, media_id) key used to dedupe candidates.
fn key_of(row: &WatchedRow) -> MediaKey {
    (row.provider.clone(), row.media_id.clone())
}

#[derive(Serialize)]
pub struct RecommendationView {
    provider: ProviderId,
    id: String,
    title: String,
    year: Option<i32>,
    thumbnail: Option<String>,
    // Exposed for display ("Because you watched X" / "Watched by N
    // friends"), not just for sorting - explainability is part of the system.
    based_on_title: Option<String>,
    co_watched_by: i64,
    score: f64,
}

// Builds this user's personalized recommendation list: collaborative
// filtering first, topped up with a content-scored catalogue fallback
// when sparse, then ranked by the blended hybrid score.
pub async fn list(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<Vec<RecommendationView>>, ApiError>
{
    let my_items: Vec<WatchedRow> = sqlx::query_as(
        "SELECT provider, media_id, title, year, thumbnail_url
         FROM watch_history WHERE user_id = $1
         ORDER BY watched_at DESC LIMIT $2",
    )
    .bind(auth.user_id)
    .bind(MAX_HISTORY)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    // No history: honestly nothing to recommend, no falling back to
    // "most watched" which would be off-topic for this module (subject chap. IV).
    if my_items.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let my_providers: Vec<String> = my_items.iter().map(|r| r.provider.clone()).collect();
    let my_media_ids: Vec<String> = my_items.iter().map(|r| r.media_id.clone()).collect();

    // --- Collaborative filtering: who else watched at least one of my
    // videos, and what proportion of my history that represents (= how much
    // their taste overlaps with mine).
    let neighbors: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT wh.user_id, COUNT(*) AS overlap
         FROM watch_history wh
         WHERE wh.user_id <> $1
           AND EXISTS (
             SELECT 1 FROM UNNEST($2::text[], $3::text[]) AS mine(provider, media_id)
             WHERE mine.provider = wh.provider AND mine.media_id = wh.media_id
           )
         GROUP BY wh.user_id",
    )
    .bind(auth.user_id)
    .bind(&my_providers)
    .bind(&my_media_ids)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    let neighbor_weight: HashMap<i64, f64> = neighbors
        .iter()
        .map(|(id, overlap)| (*id, *overlap as f64 / my_items.len() as f64))
        .collect();

    let mut candidates: HashMap<MediaKey, WatchedRow> = HashMap::new();
    let mut collab_weight: HashMap<MediaKey, f64> = HashMap::new();
    let mut collab_count: HashMap<MediaKey, i64> = HashMap::new();

    if !neighbor_weight.is_empty() {
        let neighbor_ids: Vec<i64> = neighbor_weight.keys().copied().collect();

        let neighbor_rows: Vec<(i64, WatchedRow)> = sqlx::query_as::<
            _,
            (i64, String, String, String, Option<i32>, Option<String>),
        >(
            "SELECT wh.user_id, wh.provider, wh.media_id, wh.title, wh.year, wh.thumbnail_url
             FROM watch_history wh
             WHERE wh.user_id = ANY($1::bigint[])
               AND NOT EXISTS (
                 SELECT 1 FROM UNNEST($2::text[], $3::text[]) AS mine(provider, media_id)
                 WHERE mine.provider = wh.provider AND mine.media_id = wh.media_id
               )",
        )
        .bind(&neighbor_ids)
        .bind(&my_providers)
        .bind(&my_media_ids)
        .fetch_all(&pool)
        .await
        .map_err(|_| ApiError::Internal)?
        .into_iter()
        .map(|(uid, provider, media_id, title, year, thumbnail_url)| {
            (uid, WatchedRow { provider, media_id, title, year, thumbnail_url })
        })
        .collect();

        for (neighbor_id, row) in neighbor_rows {
            let key = key_of(&row);
            let weight = neighbor_weight.get(&neighbor_id).copied().unwrap_or(0.0);
            *collab_weight.entry(key.clone()).or_insert(0.0) += weight;
            *collab_count.entry(key.clone()).or_insert(0) += 1;
            candidates.entry(key).or_insert(row);
        }
    }

    // --- Catalogue fallback: top up with everything anyone has ever
    // watched if collaborative filtering didn't find enough material (no
    // neighbors, or neighbors with little shared history). These
    // candidates enter with a zero collaborative score: content alone
    // decides whether they survive - this is the "hybrid" part of the system.
    if candidates.len() < MIN_CANDIDATES_BEFORE_CATALOG_FALLBACK {
        let catalog: Vec<WatchedRow> = sqlx::query_as(
            "SELECT DISTINCT ON (provider, media_id) provider, media_id, title, year, thumbnail_url
             FROM watch_history wh
             WHERE NOT EXISTS (
               SELECT 1 FROM UNNEST($1::text[], $2::text[]) AS mine(provider, media_id)
               WHERE mine.provider = wh.provider AND mine.media_id = wh.media_id
             )",
        )
        .bind(&my_providers)
        .bind(&my_media_ids)
        .fetch_all(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

        for row in catalog {
            candidates.entry(key_of(&row)).or_insert(row);
        }
    }

    if candidates.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // Corpus for the IDF table: my history plus the current candidates,
    // which is enough at this scale (see scoring::TfIdfModel).
    let corpus = my_items.iter().map(|r| r.title.as_str()).chain(candidates.values().map(|r| r.title.as_str()));
    let model = TfIdfModel::fit(corpus);

    let mut scored: Vec<RecommendationView> = candidates
        .into_values()
        .map(|row| {
            let key = key_of(&row);
            let collab = collab_weight.get(&key).copied().unwrap_or(0.0).min(1.0);

            let (content, based_on) = my_items
                .iter()
                .map(|mine| (model.similarity(&row.title, &mine.title), mine.title.as_str()))
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap_or((0.0, ""));

            let score = COLLAB_WEIGHT * collab + CONTENT_WEIGHT * content;
            let provider = if row.provider == ProviderId::Peertube.as_str() {
                ProviderId::Peertube
            } else {
                ProviderId::Archive
            };

            RecommendationView {
                provider,
                id: row.media_id.clone(),
                title: row.title,
                year: row.year,
                thumbnail: row.thumbnail_url,
                based_on_title: (content > 0.0).then(|| based_on.to_string()),
                co_watched_by: collab_count.get(&key).copied().unwrap_or(0),
                score,
            }
        })
        .collect();

    // The catalogue fallback above can bring in candidates with no signal
    // at all (neither co-watching nor content similarity): showing them
    // would be just as dishonest as the "most watched" fallback we
    // deliberately avoided above.
    scored.retain(|r| r.score > 0.0);
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    scored.truncate(MAX_RESULTS);

    Ok(Json(scored))
}
