// Google : OpenID Connect. https://developers.google.com/identity/protocols/oauth2/web-server

use reqwest::Url;
use serde::Deserialize;

use super::{Credentials, OAuthError, OAuthProfile};

const AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfo {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
}

pub fn authorize_url(creds: &Credentials, redirect_uri: &str, state: &str)
    -> Url
{
    Url::parse_with_params(
        AUTHORIZE_URL,
        &[
            ("client_id", creds.client_id.as_str()),
            ("redirect_uri", redirect_uri),
            ("response_type", "code"),
            ("scope", "openid email profile"),
            // Affiche toujours l'ecran "Choisir un compte", au lieu de
            // reconnecter d'office le compte deja ouvert dans le navigateur.
            ("prompt", "select_account"),
            ("state", state),
        ],
    )
    .expect("hardcoded base URL is valid")
}

pub async fn fetch_profile(http: &reqwest::Client, creds: &Credentials, redirect_uri: &str, code: &str)
    -> Result<OAuthProfile, OAuthError>
{
    let token: TokenResponse = http
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", creds.client_id.as_str()),
            ("client_secret", creds.client_secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let info: UserInfo = http
        .get(USERINFO_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let email = match info.email {
        Some(email) if info.email_verified => email,
        _ => return Err(OAuthError::EmailNotVerified),
    };

    // Pseudo propose : le nom affiche, sinon la partie avant le @.
    let username_hint = info
        .name
        .unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_string());

    Ok(OAuthProfile {
        provider_user_id: info.sub,
        email,
        username_hint,
    })
}
