// GitHub-specific OAuth wiring (OAuth App).
// https://docs.github.com/en/apps/oauth-apps/building-oauth-apps

use reqwest::Url;
use serde::Deserialize;

use super::{Credentials, OAuthError, OAuthProfile};

const AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";
const EMAILS_URL: &str = "https://api.github.com/user/emails";
const ACCEPT_JSON: &str = "application/vnd.github+json";

// GitHub answers 200 even on failure, with an `error` field instead of the token.
#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct User {
    id: u64,
    login: String,
}

#[derive(Deserialize)]
struct Email {
    email: String,
    primary: bool,
    verified: bool,
}

// Builds the URL the browser is redirected to, to start the GitHub consent screen.
pub fn authorize_url(creds: &Credentials, redirect_uri: &str, state: &str)
    -> Url
{
    Url::parse_with_params(
        AUTHORIZE_URL,
        &[
            ("client_id", creds.client_id.as_str()),
            ("redirect_uri", redirect_uri),
            // user:email: the email is often hidden on the public profile,
            // this scope is needed to read the list of verified emails.
            ("scope", "read:user user:email"),
            // Always shows the account picker, so the user can log in with
            // an account other than the one already open in the browser.
            ("prompt", "select_account"),
            ("state", state),
        ],
    )
    .expect("hardcoded base URL is valid")
}

// Exchanges the authorization code for an access token, then fetches the
// user's profile and primary verified email.
pub async fn fetch_profile(http: &reqwest::Client, creds: &Credentials, redirect_uri: &str, code: &str)
    -> Result<OAuthProfile, OAuthError>
{
    let token: TokenResponse = http
        .post(TOKEN_URL)
        .header("Accept", "application/json")
        .form(&[
            ("code", code),
            ("client_id", creds.client_id.as_str()),
            ("client_secret", creds.client_secret.as_str()),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let access_token = token.access_token.ok_or_else(|| {
        OAuthError::Provider(token.error_description.unwrap_or_else(|| "no access token returned".into()))
    })?;

    let user: User = http
        .get(USER_URL)
        .header("Accept", ACCEPT_JSON)
        .bearer_auth(&access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let emails: Vec<Email> = http
        .get(EMAILS_URL)
        .header("Accept", ACCEPT_JSON)
        .bearer_auth(&access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let email = emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
        .ok_or(OAuthError::EmailNotVerified)?;

    Ok(OAuthProfile {
        provider_user_id: user.id.to_string(),
        email,
        username_hint: user.login,
    })
}
