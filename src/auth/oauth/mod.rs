// Login via Google or GitHub: OAuth 2.0 "authorization code" flow.
//
//   1. /auth/oauth/{provider}/start    -> redirect the browser to the provider
//   2. the user approves on Google/GitHub
//   3. /auth/oauth/{provider}/callback -> the provider sends back a `code`
//   4. the server exchanges that code (+ its secret) for the user's profile
//   5. we create/find the account, open a session, redirect back to the frontend
//
// The client secret never leaves the server: that's the whole point of this flow.

mod github;
mod google;
mod handlers;

use std::fmt;
use std::time::Duration;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub use handlers::{callback, providers, start};

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    Github,
}

impl Provider {
    // Lowercase identifier used in URLs and config lookups.
    fn as_str(self)
        -> &'static str
    {
        match self {
            Provider::Google => "google",
            Provider::Github => "github",
        }
    }
}

// Credentials obtained by registering the application with the provider.
#[derive(Clone)]
pub struct Credentials {
    client_id: String,
    client_secret: String,
}

// What we keep from an external profile, the same shape regardless of provider.
pub struct OAuthProfile {
    // Stable account identifier on the provider's side (never changes).
    provider_user_id: String,
    // Email the provider guarantees has been verified.
    email: String,
    username_hint: String,
}

#[derive(Debug)]
pub enum OAuthError {
    Http(reqwest::Error),
    // The provider responded, but not with what we expected (invalid or expired code...).
    Provider(String),
    // Without a verified email we refuse: otherwise anyone could claim someone
    // else's email address and take over their SyncWatch account.
    EmailNotVerified,
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)
        -> fmt::Result
    {
        match self {
            OAuthError::Http(err) => write!(f, "http error: {err}"),
            OAuthError::Provider(msg) => write!(f, "provider error: {msg}"),
            OAuthError::EmailNotVerified => write!(f, "no verified email on the provider account"),
        }
    }
}

impl From<reqwest::Error> for OAuthError {
    fn from(err: reqwest::Error)
        -> Self
    {
        OAuthError::Http(err)
    }
}

// Shared between handlers via Extension, like MediaClient.
#[derive(Clone)]
pub struct OAuthConfig {
    http: reqwest::Client,
    backend_url: String,
    // Allowed frontend origins ("http://localhost:5173"), never empty.
    frontend_origins: Vec<String>,
    google: Option<Credentials>,
    github: Option<Credentials>,
}

impl OAuthConfig {
    // Builds the shared OAuth config from the app's Config: the HTTP client,
    // the allowed frontend origins, and whichever providers have credentials set.
    pub fn new(config: &Config)
        -> Self
    {
        // GitHub refuses requests with no User-Agent.
        let http = reqwest::Client::builder()
            .user_agent(concat!("ft_transcendence/", env!("CARGO_PKG_VERSION")))
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("failed to build HTTP client");

        let credentials = |id: &Option<String>, secret: &Option<String>| match (id, secret) {
            (Some(client_id), Some(client_secret)) => Some(Credentials {
                client_id: client_id.clone(),
                client_secret: client_secret.clone(),
            }),
            _ => None,
        };

        let frontend_origins: Vec<String> = config
            .frontend_urls
            .split(',')
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(|url| origin_of(url).expect("FRONTEND_URL must contain valid URLs"))
            .collect();
        assert!(!frontend_origins.is_empty(), "FRONTEND_URL must contain at least one URL");

        OAuthConfig {
            http,
            backend_url: config.backend_url.trim_end_matches('/').to_string(),
            frontend_origins,
            google: credentials(&config.google_client_id, &config.google_client_secret),
            github: credentials(&config.github_client_id, &config.github_client_secret),
        }
    }

    // None if the provider isn't configured in .env: it's then disabled.
    fn credentials(&self, provider: Provider)
        -> Option<&Credentials>
    {
        match provider {
            Provider::Google => self.google.as_ref(),
            Provider::Github => self.github.as_ref(),
        }
    }

    // Which frontend to send the browser back to. `candidate` comes from the
    // request (Referer header, cookie): it's only followed if it's in the
    // allow-list. Otherwise any site could get a session token delivered to
    // it (open redirect).
    fn frontend_for(&self, candidate: Option<&str>)
        -> &str
    {
        candidate
            .and_then(origin_of)
            .and_then(|origin| self.frontend_origins.iter().find(|allowed| **allowed == origin))
            .unwrap_or(&self.frontend_origins[0])
    }

    // Must be IDENTICAL, character for character, to the URL registered with the provider.
    fn redirect_uri(&self, provider: Provider)
        -> String
    {
        format!("{}/auth/oauth/{}/callback", self.backend_url, provider.as_str())
    }
}

// "http://localhost:8080/login?x=1" -> "http://localhost:8080"
fn origin_of(url: &str)
    -> Option<String>
{
    let origin = Url::parse(url).ok()?.origin();
    // An "opaque" origin (data:, file:...) names no actual site.
    origin.is_tuple().then(|| origin.ascii_serialization())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Builds a minimal OAuthConfig for testing `frontend_for` in isolation.
    fn config(frontends: &[&str])
        -> OAuthConfig
    {
        OAuthConfig {
            http: reqwest::Client::new(),
            backend_url: "http://localhost:3000".into(),
            frontend_origins: frontends.iter().map(|url| url.to_string()).collect(),
            google: None,
            github: None,
        }
    }

    // Allowed origins are followed as-is; anything else (including
    // look-alike or malformed URLs) must fall back to the first allowed one.
    #[test]
    fn frontend_for_only_follows_allowed_origins()
    {
        let config = config(&["http://localhost:5173", "http://localhost:8080"]);

        // Allowed origin: redirected there, regardless of path.
        assert_eq!(config.frontend_for(Some("http://localhost:8080/login?x=1")), "http://localhost:8080");
        assert_eq!(config.frontend_for(Some("http://localhost:5173/")), "http://localhost:5173");

        // Everything else falls back to the first address in the list.
        for hostile in [
            "https://evil.example/login",
            "http://localhost:8080.evil.example/",
            "http://localhost:9999/",
            "https://localhost:8080/",
            "javascript:alert(1)",
            "not a url",
        ] {
            assert_eq!(config.frontend_for(Some(hostile)), "http://localhost:5173", "{hostile}");
        }
        assert_eq!(config.frontend_for(None), "http://localhost:5173");
    }
}
