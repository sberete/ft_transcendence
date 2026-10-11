// Source : une instance PeerTube (plateforme video libre et federee).
// API REST publique : https://docs.joinpeertube.org/api-rest-reference.html
//
// Securite (SSRF) : le serveur n'appelle QUE l'instance configuree dans
// PEERTUBE_URL. Le client envoie un uuid, jamais une URL ni un hote ; sinon il
// pourrait faire interroger a notre backend n'importe quelle adresse interne.

use serde::Deserialize;

use super::{MediaClient, MediaDetails, MediaItem, ProviderId, StreamKind, StreamLink, MAX_RESULTS_PER_PROVIDER};

#[derive(Deserialize)]
struct SearchResponse {
    data: Vec<VideoSummary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoSummary {
    uuid: String,
    name: String,
    duration: Option<u64>,
    thumbnail_path: Option<String>,
    published_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoDetails {
    uuid: String,
    name: String,
    description: Option<String>,
    duration: Option<u64>,
    thumbnail_path: Option<String>,
    preview_path: Option<String>,
    published_at: Option<String>,
    #[serde(default)]
    files: Vec<VideoFile>,
    #[serde(default)]
    streaming_playlists: Vec<StreamingPlaylist>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoFile {
    file_url: String,
    resolution: Resolution,
}

#[derive(Deserialize)]
struct Resolution {
    label: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamingPlaylist {
    playlist_url: String,
}

// Format uuid v4 : 36 caracteres hexa et tirets.
pub fn is_valid_id(id: &str)
    -> bool
{
    id.len() == 36 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

pub async fn search(client: &MediaClient, query: &str)
    -> Result<Vec<MediaItem>, reqwest::Error>
{
    let url = client.peertube_base.join("/api/v1/search/videos").expect("static path is valid");
    let count = MAX_RESULTS_PER_PROVIDER.to_string();

    let request = client
        .http
        .get(url)
        .query(&[
            ("search", query),
            ("count", count.as_str()),
            // "local" : seulement les videos hebergees sur l'instance, pour que
            // les flux viennent bien de la source qu'on a choisi de faire confiance.
            ("searchTarget", "local"),
            ("nsfw", "false"),
        ]);
    let body: SearchResponse = client.get_json(request).await?;

    let items = body
        .data
        .into_iter()
        .filter(|video| is_valid_id(&video.uuid))
        .map(|video| MediaItem {
            provider: ProviderId::Peertube,
            title: video.name,
            year: year_of(video.published_at.as_deref()),
            thumbnail: asset_url(client, video.thumbnail_path.as_deref()),
            duration_seconds: video.duration,
            id: video.uuid,
        })
        .collect();

    Ok(items)
}

pub async fn details(client: &MediaClient, uuid: &str)
    -> Result<MediaDetails, reqwest::Error>
{
    let url = client
        .peertube_base
        .join(&format!("/api/v1/videos/{uuid}"))
        .expect("uuid was validated");

    let request = client
        .http
        .get(url);
    let video: VideoDetails = client.get_json(request).await?;

    // HLS en premier : une seule playlist "master" contient toutes les qualites,
    // le lecteur choisit automatiquement selon la bande passante.
    let hls = video.streaming_playlists.into_iter().map(|playlist| StreamLink {
        kind: StreamKind::Hls,
        url: playlist.playlist_url,
        quality: Some("auto".into()),
    });

    let mp4 = video.files.into_iter().map(|file| StreamLink {
        kind: StreamKind::Mp4,
        url: file.file_url,
        quality: Some(file.resolution.label),
    });

    Ok(MediaDetails {
        item: MediaItem {
            provider: ProviderId::Peertube,
            id: video.uuid,
            title: video.name,
            year: year_of(video.published_at.as_deref()),
            thumbnail: asset_url(client, video.thumbnail_path.as_deref()),
            duration_seconds: video.duration,
        },
        description: video.description,
        backdrop: asset_url(client, video.preview_path.as_deref()),
        streams: hls.chain(mp4).collect(),
    })
}

// PeerTube renvoie des chemins relatifs ("/lazy-static/...") : on les rattache a l'instance.
fn asset_url(client: &MediaClient, path: Option<&str>)
    -> Option<String>
{
    path.and_then(|p| client.peertube_base.join(p).ok()).map(|url| url.to_string())
}

fn year_of(date: Option<&str>)
    -> Option<i32>
{
    date.and_then(|d| d.get(..4)).and_then(|y| y.parse().ok())
}
