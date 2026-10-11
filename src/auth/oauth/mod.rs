// Connexion via Google ou GitHub : OAuth 2.0, flux "authorization code".
//
//   1. /auth/oauth/{provider}/start    -> on redirige le navigateur chez le fournisseur
//   2. l'utilisateur accepte chez Google/GitHub
//   3. /auth/oauth/{provider}/callback -> le fournisseur nous renvoie un `code`
//   4. le serveur echange ce code (+ son secret) contre le profil de l'utilisateur
//   5. on cree/retrouve le compte, on ouvre une session, on renvoie vers le front
//
// Le secret client ne quitte jamais le serveur : c'est tout l'interet de ce flux.

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
    fn as_str(self)
        -> &'static str
    {
        match self {
            Provider::Google => "google",
            Provider::Github => "github",
        }
    }
}

// Identifiants obtenus en enregistrant l'application chez le fournisseur.
#[derive(Clone)]
pub struct Credentials {
    client_id: String,
    client_secret: String,
}

// Ce qu'on retient d'un profil externe, identique quel que soit le fournisseur.
pub struct OAuthProfile {
    // Identifiant stable du compte chez le fournisseur (ne change jamais).
    provider_user_id: String,
    // Email dont le fournisseur garantit qu'il a ete verifie.
    email: String,
    username_hint: String,
}

#[derive(Debug)]
pub enum OAuthError {
    Http(reqwest::Error),
    // Le fournisseur a repondu, mais pas ce qu'on attendait (code invalide, expire...).
    Provider(String),
    // Sans email verifie on refuse : sinon n'importe qui pourrait declarer
    // l'email d'un autre et recuperer son compte SyncWatch.
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

// Partage entre les handlers via Extension, comme MediaClient.
#[derive(Clone)]
pub struct OAuthConfig {
    http: reqwest::Client,
    backend_url: String,
    // Origines du front autorisees ("http://localhost:5173"), jamais vide.
    frontend_origins: Vec<String>,
    google: Option<Credentials>,
    github: Option<Credentials>,
}

impl OAuthConfig {
    pub fn new(config: &Config)
        -> Self
    {
        // GitHub refuse les requetes sans User-Agent.
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

    // None si le fournisseur n'est pas configure dans .env : il est alors desactive.
    fn credentials(&self, provider: Provider)
        -> Option<&Credentials>
    {
        match provider {
            Provider::Google => self.google.as_ref(),
            Provider::Github => self.github.as_ref(),
        }
    }

    // Vers quel front renvoyer le navigateur. `candidate` vient de la requete
    // (en-tete Referer, cookie) : on ne le suit QUE s'il figure dans la liste
    // autorisee. Sinon, n'importe quel site pourrait se faire livrer un token
    // de session (open redirect).
    fn frontend_for(&self, candidate: Option<&str>)
        -> &str
    {
        candidate
            .and_then(origin_of)
            .and_then(|origin| self.frontend_origins.iter().find(|allowed| **allowed == origin))
            .unwrap_or(&self.frontend_origins[0])
    }

    // Doit etre IDENTIQUE a l'URL declaree chez le fournisseur, au caractere pres.
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
    // Une origine "opaque" (data:, file:...) ne designe aucun site.
    origin.is_tuple().then(|| origin.ascii_serialization())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn frontend_for_only_follows_allowed_origins()
    {
        let config = config(&["http://localhost:5173", "http://localhost:8080"]);

        // Origine autorisee : on y retourne, quel que soit le chemin.
        assert_eq!(config.frontend_for(Some("http://localhost:8080/login?x=1")), "http://localhost:8080");
        assert_eq!(config.frontend_for(Some("http://localhost:5173/")), "http://localhost:5173");

        // Tout le reste retombe sur la premiere adresse de la liste.
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
