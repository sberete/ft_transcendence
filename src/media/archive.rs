// Source : Internet Archive, collection "feature_films" (films du domaine public).
// API publique, sans cle : https://archive.org/developers/

use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;

use super::{MediaClient, MediaDetails, MediaItem, ProviderId, StreamKind, StreamLink, MAX_RESULTS_PER_PROVIDER};

const SEARCH_URL: &str = "https://archive.org/advancedsearch.php";
const METADATA_URL: &str = "https://archive.org/metadata/";
const DOWNLOAD_URL: &str = "https://archive.org/download/";
const THUMBNAIL_URL: &str = "https://archive.org/services/img/";

#[derive(Deserialize)]
struct SearchResponse {
    response: SearchDocs,
}

#[derive(Deserialize)]
struct SearchDocs {
    docs: Vec<SearchDoc>,
}

// `title` et `year` sont parfois des chaines, parfois des nombres ou des tableaux :
// on les garde en Value et on les normalise nous-memes.
#[derive(Deserialize)]
struct SearchDoc {
    identifier: String,
    title: Option<Value>,
    year: Option<Value>,
}

#[derive(Deserialize)]
struct MetadataResponse {
    #[serde(default)]
    metadata: ItemMetadata,
    #[serde(default)]
    files: Vec<MetadataFile>,
}

#[derive(Deserialize, Default)]
struct ItemMetadata {
    title: Option<Value>,
    description: Option<Value>,
    year: Option<Value>,
    date: Option<Value>,
}

#[derive(Deserialize)]
struct MetadataFile {
    name: String,
    format: Option<String>,
    height: Option<String>,
}

// Les identifiants Archive ne contiennent que [A-Za-z0-9._-].
// Les valider empeche d'injecter "../" ou une autre URL dans le chemin.
pub fn is_valid_id(id: &str)
    -> bool
{
    !id.is_empty()
        && id.len() <= 100
        && !id.starts_with('.')
        && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

pub async fn search(client: &MediaClient, query: &str)
    -> Result<Vec<MediaItem>, reqwest::Error>
{
    // La recherche Archive utilise la syntaxe Lucene : sans nettoyage, un
    // utilisateur pourrait ecrire `x) OR mediatype:(texts` et elargir la requete.
    // On ne garde que lettres, chiffres et espaces.
    let terms: String = query
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    let terms = terms.split_whitespace().collect::<Vec<_>>().join(" ");
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let lucene = format!("title:({terms}) AND mediatype:movies AND collection:feature_films");
    let rows = MAX_RESULTS_PER_PROVIDER.to_string();

    let request = client
        .http
        .get(SEARCH_URL)
        .query(&[
            ("q", lucene.as_str()),
            ("fl[]", "identifier"),
            ("fl[]", "title"),
            ("fl[]", "year"),
            ("rows", rows.as_str()),
            ("output", "json"),
        ]);
    let body: SearchResponse = client.get_json(request).await?;

    let items = body
        .response
        .docs
        .into_iter()
        .filter(|doc| is_valid_id(&doc.identifier))
        .map(|doc| MediaItem {
            provider: ProviderId::Archive,
            title: doc.title.as_ref().and_then(value_to_string).unwrap_or_else(|| doc.identifier.clone()),
            year: doc.year.as_ref().and_then(value_to_string).and_then(|y| y.parse().ok()),
            thumbnail: Some(format!("{THUMBNAIL_URL}{}", doc.identifier)),
            duration_seconds: None,
            id: doc.identifier,
        })
        .collect();

    Ok(items)
}

pub async fn details(client: &MediaClient, id: &str)
    -> Result<MediaDetails, reqwest::Error>
{
    let request = client
        .http
        .get(join_segment(METADATA_URL, id));
    let body: MetadataResponse = client.get_json(request).await?;

    let streams = body
        .files
        .into_iter()
        .filter(|file| {
            let is_mp4 = file.name.to_ascii_lowercase().ends_with(".mp4");
            let is_video = file.format.as_deref().is_some_and(|f| f.contains("MPEG4") || f.contains("h.264"));
            is_mp4 && is_video
        })
        .map(|file| StreamLink {
            kind: StreamKind::Mp4,
            url: join_segment(&format!("{DOWNLOAD_URL}{id}/"), &file.name).to_string(),
            quality: file.height.map(|h| format!("{h}p")),
        })
        .collect();

    let meta = body.metadata;
    // `year` est souvent absent, `date` vaut "1968" ou "1968-10-01".
    let year = meta
        .year
        .or(meta.date)
        .as_ref()
        .and_then(value_to_string)
        .and_then(|d| d.get(..4).and_then(|y| y.parse().ok()));

    Ok(MediaDetails {
        item: MediaItem {
            provider: ProviderId::Archive,
            id: id.to_string(),
            title: meta.title.as_ref().and_then(value_to_string).unwrap_or_else(|| id.to_string()),
            year,
            thumbnail: Some(format!("{THUMBNAIL_URL}{id}")),
            duration_seconds: None,
        },
        description: meta.description.as_ref().and_then(value_to_string),
        backdrop: None,
        streams,
    })
}

// Ajoute un segment de chemin en l'encodant (espaces, '#', '?'...) plutot que
// de concatener des chaines.
fn join_segment(base: &str, segment: &str)
    -> Url
{
    let mut url = Url::parse(base).expect("hardcoded base URL is valid");
    url.path_segments_mut()
        .expect("https URL can have path segments")
        .pop_if_empty()
        .push(segment);
    url
}

fn value_to_string(value: &Value)
    -> Option<String>
{
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(values) => values.first().and_then(value_to_string),
        _ => None,
    }
}
