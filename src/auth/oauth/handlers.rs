// HTTP handlers for the OAuth flow: listing enabled providers, starting
// the redirect to the provider, and handling its callback (CSRF-safe via
// a state cookie, account linking by verified email).

use axum::extract::{Path, Query, State};
use axum::http::header::{COOKIE, REFERER, SET_COOKIE};
use axum::http::HeaderMap;
use axum::response::{AppendHeaders, IntoResponse, Redirect, Response};
use axum::{Extension, Json};
use rand::RngCore;
use serde::Deserialize;
use sqlx::PgPool;

use super::{github, google, OAuthConfig, OAuthError, OAuthProfile, Provider};
use crate::auth::session;

const STATE_COOKIE: &str = "oauth_state";
// Remembers which frontend the login was started from (Vite or nginx).
const RETURN_COOKIE: &str = "oauth_return";
const COOKIE_TTL_SECONDS: u32 = 600;
const MAX_USERNAME_BASE_LEN: usize = 24;
const USERNAME_ATTEMPTS: usize = 5;

#[derive(Deserialize)]
pub struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    // Present if the user clicked "Deny" on the provider's side.
    error: Option<String>,
}

// Why a callback failed. Never shown to the user, only logged.
enum CallbackError {
    NotConfigured,
    Denied(String),
    StateMismatch,
    MissingCode,
    Provider(OAuthError),
    Database(sqlx::Error),
}

// Lists the providers that are actually enabled (configured in .env).
pub async fn providers(Extension(config): Extension<OAuthConfig>)
    -> Json<Vec<Provider>>
{
    let enabled = [Provider::Google, Provider::Github]
        .into_iter()
        .filter(|provider| config.credentials(*provider).is_some())
        .collect();

    Json(enabled)
}

// Redirects the browser to the provider's consent screen, after setting
// a CSRF state cookie and remembering which frontend to return to.
pub async fn start(
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Extension(config): Extension<OAuthConfig>,
)
    -> Response
{
    // We'll send the browser back where it came from (checked against the allow-list).
    let referer = headers.get(REFERER).and_then(|value| value.to_str().ok());
    let frontend = config.frontend_for(referer);

    // The button is always shown on the frontend. If the provider isn't
    // configured in .env, redirect back to the frontend with an error code,
    // rather than showing a raw JSON error in the browser.
    let Some(creds) = config.credentials(provider) else {
        log::warn!("oauth start for {provider:?} but its client id/secret are not set in the environment");
        return Redirect::to(&format!("{frontend}/oauth/callback?error=oauth_unavailable")).into_response();
    };

    // `state`: a random value sent to the provider AND stored in a cookie.
    // On return, both must match. Without this, an attacker could get their
    // victim to open a callback containing THEIR OWN code, logging the
    // victim into the attacker's account without realizing it (login CSRF).
    let state = random_state();
    let redirect_uri = config.redirect_uri(provider);

    let url = match provider {
        Provider::Google => google::authorize_url(creds, &redirect_uri, &state),
        Provider::Github => github::authorize_url(creds, &redirect_uri, &state),
    };

    // AppendHeaders: two Set-Cookie headers, the second doesn't overwrite the first.
    let cookies = AppendHeaders([
        (SET_COOKIE, cookie(&config, STATE_COOKIE, &state, COOKIE_TTL_SECONDS)),
        (SET_COOKIE, cookie(&config, RETURN_COOKIE, frontend, COOKIE_TTL_SECONDS)),
    ]);
    (cookies, Redirect::to(url.as_str())).into_response()
}

// Called by the browser when the provider redirects back: always answers
// with a redirect to the frontend, never with raw JSON.
pub async fn callback(
    Path(provider): Path<Provider>,
    Query(params): Query<CallbackParams>,
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Extension(config): Extension<OAuthConfig>,
)
    -> Response
{
    // A cookie can be edited by the client: its value is re-validated here.
    let return_cookie = read_cookie(&headers, RETURN_COOKIE);
    let frontend = config.frontend_for(return_cookie.as_deref());

    let target = match finish_login(provider, params, &headers, &pool, &config).await {
        // The token is placed in the fragment (#...): this part of the URL
        // is never sent to a server, so it never ends up in access logs.
        Ok(token) => format!("{frontend}/oauth/callback#token={token}"),
        Err(err) => {
            log::warn!("oauth callback failed for {provider:?}: {}", describe(&err));
            format!("{frontend}/oauth/callback?error=oauth")
        }
    };

    // These cookies are single-use: clear them in every case.
    let cookies = AppendHeaders([
        (SET_COOKIE, cookie(&config, STATE_COOKIE, "", 0)),
        (SET_COOKIE, cookie(&config, RETURN_COOKIE, "", 0)),
    ]);
    (cookies, Redirect::to(&target)).into_response()
}

// Validates the callback (state, error, code), exchanges the code for a
// profile with the provider, resolves/creates the local account, and
// opens a session for it.
async fn finish_login(
    provider: Provider,
    params: CallbackParams,
    headers: &HeaderMap,
    pool: &PgPool,
    config: &OAuthConfig,
)
    -> Result<String, CallbackError>
{
    let creds = config.credentials(provider).ok_or(CallbackError::NotConfigured)?;

    if let Some(error) = params.error {
        return Err(CallbackError::Denied(error));
    }

    let expected = read_cookie(headers, STATE_COOKIE).ok_or(CallbackError::StateMismatch)?;
    match params.state {
        Some(state) if state == expected => {}
        _ => return Err(CallbackError::StateMismatch),
    }

    let code = params.code.ok_or(CallbackError::MissingCode)?;
    let redirect_uri = config.redirect_uri(provider);

    let profile = match provider {
        Provider::Google => google::fetch_profile(&config.http, creds, &redirect_uri, &code).await,
        Provider::Github => github::fetch_profile(&config.http, creds, &redirect_uri, &code).await,
    }
    .map_err(CallbackError::Provider)?;

    let user_id = find_or_create_user(pool, provider, &profile)
        .await
        .map_err(CallbackError::Database)?;

    session::create_session(pool, user_id).await.map_err(CallbackError::Database)
}

// Resolves the local user behind an OAuth profile: an already-linked
// account, an existing account with the same (verified) email, or a
// brand new account - in that order.
async fn find_or_create_user(pool: &PgPool, provider: Provider, profile: &OAuthProfile)
    -> Result<i64, sqlx::Error>
{
    // 1. External identity already known: the normal case for a returning user.
    let linked: Option<(i64,)> =
        sqlx::query_as("SELECT user_id FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2")
            .bind(provider.as_str())
            .bind(&profile.provider_user_id)
            .fetch_optional(pool)
            .await?;
    if let Some((user_id,)) = linked {
        return Ok(user_id);
    }

    let mut tx = pool.begin().await?;

    // 2. An account already exists with this email: link the external
    //    identity to it. Only acceptable because the email is verified by the provider.
    let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE lower(email) = lower($1)")
        .bind(&profile.email)
        .fetch_optional(&mut *tx)
        .await?;

    // 3. Otherwise, a brand new account with no password.
    let user_id = match existing {
        Some((user_id,)) => user_id,
        None => create_user(&mut tx, profile).await?,
    };

    sqlx::query("INSERT INTO oauth_accounts (provider, provider_user_id, user_id) VALUES ($1, $2, $3)")
        .bind(provider.as_str())
        .bind(&profile.provider_user_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(user_id)
}

// Creates a brand new user from an OAuth profile, retrying with a random
// suffix if the suggested username is already taken.
async fn create_user(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, profile: &OAuthProfile)
    -> Result<i64, sqlx::Error>
{
    let base = sanitize_username(&profile.username_hint);

    // The suggested username might be taken: retry with a random suffix.
    // ON CONFLICT DO NOTHING rather than an error, because a SQL error
    // would abort the whole transaction currently in progress.
    for attempt in 0..USERNAME_ATTEMPTS {
        let candidate = if attempt == 0 {
            base.clone()
        } else {
            format!("{base}_{}", random_hex(2))
        };

        let created: Option<(i64,)> = sqlx::query_as(
            "INSERT INTO users (username, email) VALUES ($1, $2)
             ON CONFLICT (username) DO NOTHING
             RETURNING id",
        )
        .bind(&candidate)
        .bind(&profile.email)
        .fetch_optional(&mut **tx)
        .await?;

        if let Some((user_id,)) = created {
            return Ok(user_id);
        }
    }

    Err(sqlx::Error::RowNotFound)
}

// The name comes from the provider, i.e. from the outside: only safe
// characters are kept, and the column's VARCHAR(32) limit is respected.
fn sanitize_username(hint: &str)
    -> String
{
    let cleaned: String = hint
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(MAX_USERNAME_BASE_LEN)
        .collect();

    if cleaned.is_empty() { "user".to_string() } else { cleaned }
}

// HttpOnly: unreadable from JavaScript. SameSite=Lax: sent on the return
// navigation from the provider, but not on requests from another site.
fn cookie(config: &OAuthConfig, name: &str, value: &str, max_age: u32)
    -> String
{
    let secure = if config.backend_url.starts_with("https://") { "; Secure" } else { "" };
    // Path=/ (not /auth/oauth): behind nginx these routes are seen under
    // /api/auth/oauth (the /api prefix), so the cookie needs to cover a
    // broad path rather than depend on a prefix this module doesn't know about.
    format!("{name}={value}; Path=/; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}")
}

// Reads one named cookie out of the raw Cookie header.
fn read_cookie(headers: &HeaderMap, name: &str)
    -> Option<String>
{
    let prefix = format!("{name}=");

    headers
        .get(COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|pair| pair.trim().strip_prefix(&prefix))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

// Generates the random CSRF state value sent to the provider.
fn random_state()
    -> String
{
    random_hex(32)
}

// Generates `bytes` random bytes, hex-encoded.
fn random_hex(bytes: usize)
    -> String
{
    let mut raw = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut raw);
    hex::encode(raw)
}

// Human-readable description of a callback failure, for logging only.
fn describe(err: &CallbackError)
    -> String
{
    match err {
        CallbackError::NotConfigured => "provider not configured".into(),
        CallbackError::Denied(reason) => format!("denied by user or provider: {reason}"),
        CallbackError::StateMismatch => "state cookie missing or different (possible CSRF)".into(),
        CallbackError::MissingCode => "no authorization code in callback".into(),
        CallbackError::Provider(err) => err.to_string(),
        CallbackError::Database(err) => format!("database error: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Builds a minimal OAuthProfile for the tests below.
    fn profile(id: &str, email: &str, hint: &str)
        -> OAuthProfile
    {
        OAuthProfile {
            provider_user_id: id.to_string(),
            email: email.to_string(),
            username_hint: hint.to_string(),
        }
    }

    // Only ASCII alphanumerics and underscore survive, and the result
    // never exceeds the max base length or ends up empty.
    #[test]
    fn sanitize_username_keeps_only_safe_characters()
    {
        assert_eq!(sanitize_username("Jean Dupont"), "JeanDupont");
        assert_eq!(sanitize_username("<script>alert(1)</script>"), "scriptalert1script");
        assert_eq!(sanitize_username("été"), "t");
        assert_eq!(sanitize_username("!!!"), "user");
        assert_eq!(sanitize_username(&"a".repeat(100)).len(), MAX_USERNAME_BASE_LEN);
    }

    // Finds the right cookie among several, and never treats an emptied
    // (erased) cookie as a valid value.
    #[test]
    fn read_cookie_finds_the_right_cookie()
    {
        let mut headers = HeaderMap::new();
        assert_eq!(read_cookie(&headers, STATE_COOKIE), None);

        headers.insert(COOKIE, "theme=dark; oauth_state=abc123; other=1".parse().unwrap());
        assert_eq!(read_cookie(&headers, STATE_COOKIE), Some("abc123".to_string()));

        // An empty cookie (after being cleared) must never validate an empty state.
        headers.insert(COOKIE, "oauth_state=".parse().unwrap());
        assert_eq!(read_cookie(&headers, STATE_COOKIE), None);
    }

    // Integration test: needs the real database (make dev).
    //   cargo test -- --ignored
    #[tokio::test]
    #[ignore = "requires the PostgreSQL database from DATABASE_URL"]
    async fn find_or_create_user_links_accounts()
    {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = crate::db::init_pool(&url).await.expect("database unreachable");
        crate::db::run_migrations(&pool).await.expect("migrations failed");

        let cleanup = "DELETE FROM users WHERE email LIKE '%@oauth-test.invalid'";
        sqlx::query(cleanup).execute(&pool).await.unwrap();

        // First login: the account is created, with no password.
        let alice = profile("g-1", "alice@oauth-test.invalid", "Oauth Test Alice");
        let id = find_or_create_user(&pool, Provider::Google, &alice).await.unwrap();
        let row: (String, Option<String>) =
            sqlx::query_as("SELECT username, password_hash FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row, ("OauthTestAlice".to_string(), None));

        // Returning login: same account, even if the email changed on the provider's side.
        let renamed = profile("g-1", "other@oauth-test.invalid", "Whatever");
        assert_eq!(find_or_create_user(&pool, Provider::Google, &renamed).await.unwrap(), id);

        // Different provider, same email (different case): linked to the same account.
        let via_github = profile("gh-9", "ALICE@oauth-test.invalid", "alice-gh");
        assert_eq!(find_or_create_user(&pool, Provider::Github, &via_github).await.unwrap(), id);

        // Different person, same suggested username: a new account with a suffix.
        let twin = profile("g-2", "twin@oauth-test.invalid", "Oauth Test Alice");
        let twin_id = find_or_create_user(&pool, Provider::Google, &twin).await.unwrap();
        assert_ne!(twin_id, id);
        let (twin_name,): (String,) = sqlx::query_as("SELECT username FROM users WHERE id = $1")
            .bind(twin_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(twin_name.starts_with("OauthTestAlice_"), "got {twin_name}");

        sqlx::query(cleanup).execute(&pool).await.unwrap();
    }
}
