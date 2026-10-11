// Agregateur de videos legales, sur le meme principe que Movix :
// on n'heberge rien, on interroge des sources externes et on renvoie
// au front des liens directs (MP4 / HLS) qu'il lit lui-meme.
//
// Difference avec Movix : uniquement des sources qui l'autorisent,
// via leur API publique, sans contournement de protection.

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

// archive.org repond parfois en plus de 10s : en dessous, trop de faux echecs.
const HTTP_TIMEOUT: Duration = Duration::from_secs(12);
const RETRY_DELAY: Duration = Duration::from_millis(500);
const MAX_RESULTS_PER_PROVIDER: usize = 12;

pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/search", get(handlers::search))
        .route("/{provider}/{id}", get(handlers::details))
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Archive,
    Peertube,
}

// Un resultat de recherche, identique quelle que soit la source.
#[derive(Serialize)]
pub struct MediaItem {
    provider: ProviderId,
    id: String,
    title: String,
    year: Option<i32>,
    thumbnail: Option<String>,
    duration_seconds: Option<u64>,
}

// Fiche complete d'une video : ce qu'affiche la page de lecture.
#[derive(Serialize)]
pub struct MediaDetails {
    #[serde(flatten)]
    item: MediaItem,
    // Peut contenir du HTML (Archive) : le front doit l'afficher en texte brut.
    description: Option<String>,
    // Image plus grande que `thumbnail`, pour le fond de la page.
    backdrop: Option<String>,
    streams: Vec<StreamLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    Mp4,
    Hls,
}

// Un lien lisible directement par le lecteur du front.
#[derive(Serialize)]
pub struct StreamLink {
    kind: StreamKind,
    url: String,
    quality: Option<String>,
}

// Partage entre les handlers via Extension (reqwest::Client est un Arc en interne,
// le cloner ne coute rien et reutilise le pool de connexions).
#[derive(Clone)]
pub struct MediaClient {
    http: reqwest::Client,
    peertube_base: Url,
}

impl MediaClient {
    pub fn new(peertube_url: &str)
        -> Self
    {
        // User-Agent honnete : on s'identifie au lieu d'imiter un navigateur.
        let http = reqwest::Client::builder()
            .user_agent(concat!("ft_transcendence/", env!("CARGO_PKG_VERSION")))
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("failed to build HTTP client");

        let peertube_base = Url::parse(peertube_url).expect("PEERTUBE_URL must be a valid URL");

        MediaClient { http, peertube_base }
    }

    // GET + decodage JSON, avec UNE nouvelle tentative si l'echec est passager
    // (timeout, connexion, 5xx) : archive.org renvoie regulierement des 502 isoles.
    // Pas plus d'une : on ne veut pas marteler une source deja en difficulte.
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

async fn send_json<T: DeserializeOwned>(request: RequestBuilder)
    -> Result<T, reqwest::Error>
{
    request.send().await?.error_for_status()?.json().await
}

fn is_transient(err: &reqwest::Error)
    -> bool
{
    err.is_timeout() || err.is_connect() || err.status().is_some_and(|s| s.is_server_error())
}
