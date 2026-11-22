// Source: a PeerTube instance (free, federated video platform).
// Public REST API: https://docs.joinpeertube.org/api-rest-reference.html
//
// Security (SSRF): the server ONLY calls the instance configured in
// PEERTUBE_URL. The client sends a uuid, never a URL or a host;
// otherwise it could make our backend query any internal address.

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

// uuid v4 format: 36 characters of hex digits and dashes.
pub fn is_valid_id(id: &str)
    -> bool
{
    id.len() == 36 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

// Searches the configured instance by title and maps the results to
// our common MediaItem shape.
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
            // "local": only videos hosted on this instance, so streams
            // actually come from the source we chose to trust.
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

// Fetches a video's details and builds its list of playable streams
// (HLS master playlist plus any direct MP4 files).
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

    // HLS first: a single "master" playlist holds every quality, the
    // player picks automatically based on bandwidth.
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

// PeerTube returns relative paths ("/lazy-static/..."): resolved against the instance's base URL.
fn asset_url(client: &MediaClient, path: Option<&str>)
    -> Option<String>
{
    path.and_then(|p| client.peertube_base.join(p).ok()).map(|url| url.to_string())
}

// Extracts the year from an ISO date string ("2023-05-01...").
fn year_of(date: Option<&str>)
    -> Option<i32>
{
    date.and_then(|d| d.get(..4)).and_then(|y| y.parse().ok())
}
