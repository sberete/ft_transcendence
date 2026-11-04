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
// Poids du terme collaboratif dans le score final. Le terme s'annule tout
// seul (vaut 0) quand il n'y a aucun voisin partageant des gouts avec
// l'utilisateur : le contenu prend alors le relais sans bascule explicite
// (if/else) entre les deux modes - c'est ca, "hybride".
const COLLAB_WEIGHT: f64 = 0.6;
const CONTENT_WEIGHT: f64 = 1.0 - COLLAB_WEIGHT;
// Sous ce nombre de candidats trouves par filtrage collaboratif, on complete
// avec le catalogue general (tout ce que quelqu'un a deja regarde) pour
// garantir qu'il y a toujours de quoi proposer autre chose que du contenu
// deja vu, meme pour un compte sans "voisins".
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
    // Expose pour l'affichage ("Parce que vous avez regardé X" / "Vu par N amis"),
    // pas seulement pour le tri - l'explicabilite fait partie du systeme.
    based_on_title: Option<String>,
    co_watched_by: i64,
    score: f64,
}

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

    // Pas d'historique : rien a recommander honnetement, pas de repli sur
    // "les plus regardes" qui serait hors sujet pour ce module (chap. IV).
    if my_items.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let my_providers: Vec<String> = my_items.iter().map(|r| r.provider.clone()).collect();
    let my_media_ids: Vec<String> = my_items.iter().map(|r| r.media_id.clone()).collect();

    // --- Filtrage collaboratif : qui d'autre a regarde au moins une de mes
    // videos, et dans quelle proportion de mon historique (= a quel point ses
    // gouts recoupent les miens).
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

    // --- Repli "catalogue" : complete avec tout ce que quelqu'un a deja
    // regarde si le filtrage collaboratif n'a pas assez de matiere (compte
    // sans voisins, ou voisins avec peu d'historique en commun). Ces
    // candidats entrent avec un score collaboratif nul : seul le contenu les
    // departagera, c'est la partie "hybride" du systeme.
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

    // Corpus pour l'IDF : mon historique + les candidats du moment, suffisant
    // a cette echelle (voir scoring::TfIdfModel).
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

    // Le repli "catalogue" ci-dessus peut amener des candidats sans aucun
    // signal (ni co-visionnage, ni similarite de contenu) : les afficher
    // serait aussi malhonnete que le repli "les plus regardes" qu'on a
    // volontairement evite plus haut.
    scored.retain(|r| r.score > 0.0);
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    scored.truncate(MAX_RESULTS);

    Ok(Json(scored))
}
