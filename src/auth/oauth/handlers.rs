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
// Retient depuis quel front la connexion a ete lancee (Vite ou nginx).
const RETURN_COOKIE: &str = "oauth_return";
const COOKIE_TTL_SECONDS: u32 = 600;
const MAX_USERNAME_BASE_LEN: usize = 24;
const USERNAME_ATTEMPTS: usize = 5;

#[derive(Deserialize)]
pub struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    // Present si l'utilisateur a clique "Refuser" chez le fournisseur.
    error: Option<String>,
}

// Pourquoi un callback a echoue. Jamais montre a l'utilisateur, seulement logue.
enum CallbackError {
    NotConfigured,
    Denied(String),
    StateMismatch,
    MissingCode,
    Provider(OAuthError),
    Database(sqlx::Error),
}

// Liste des fournisseurs actives (configures dans .env).
pub async fn providers(Extension(config): Extension<OAuthConfig>)
    -> Json<Vec<Provider>>
{
    let enabled = [Provider::Google, Provider::Github]
        .into_iter()
        .filter(|provider| config.credentials(*provider).is_some())
        .collect();

    Json(enabled)
}

pub async fn start(
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Extension(config): Extension<OAuthConfig>,
)
    -> Response
{
    // On renverra le navigateur la d'ou il vient (verifie contre la liste autorisee).
    let referer = headers.get(REFERER).and_then(|value| value.to_str().ok());
    let frontend = config.frontend_for(referer);

    // Le bouton est toujours affiche cote front. Si le fournisseur n'est pas
    // configure dans .env, on renvoie vers le front avec un code d'erreur,
    // plutot que d'afficher une erreur JSON brute dans le navigateur.
    let Some(creds) = config.credentials(provider) else {
        log::warn!("oauth start for {provider:?} but its client id/secret are not set in the environment");
        return Redirect::to(&format!("{frontend}/oauth/callback?error=oauth_unavailable")).into_response();
    };

    // `state` : valeur aleatoire envoyee au fournisseur ET posee en cookie.
    // Au retour, les deux doivent correspondre. Sans ca, un attaquant pourrait
    // faire ouvrir a sa victime un callback contenant SON code, et la connecter
    // a son insu sur le compte de l'attaquant (CSRF de connexion).
    let state = random_state();
    let redirect_uri = config.redirect_uri(provider);

    let url = match provider {
        Provider::Google => google::authorize_url(creds, &redirect_uri, &state),
        Provider::Github => github::authorize_url(creds, &redirect_uri, &state),
    };

    // AppendHeaders : deux en-tetes Set-Cookie, le second ne remplace pas le premier.
    let cookies = AppendHeaders([
        (SET_COOKIE, cookie(&config, STATE_COOKIE, &state, COOKIE_TTL_SECONDS)),
        (SET_COOKIE, cookie(&config, RETURN_COOKIE, frontend, COOKIE_TTL_SECONDS)),
    ]);
    (cookies, Redirect::to(url.as_str())).into_response()
}

// Appele par le navigateur au retour du fournisseur : on repond toujours par
// une redirection vers le front, jamais par du JSON.
pub async fn callback(
    Path(provider): Path<Provider>,
    Query(params): Query<CallbackParams>,
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Extension(config): Extension<OAuthConfig>,
)
    -> Response
{
    // Un cookie est modifiable par le client : sa valeur est revalidee ici.
    let return_cookie = read_cookie(&headers, RETURN_COOKIE);
    let frontend = config.frontend_for(return_cookie.as_deref());

    let target = match finish_login(provider, params, &headers, &pool, &config).await {
        // Le token est place dans le fragment (#...) : cette partie de l'URL
        // n'est jamais envoyee a un serveur, donc absente des logs d'acces.
        Ok(token) => format!("{frontend}/oauth/callback#token={token}"),
        Err(err) => {
            log::warn!("oauth callback failed for {provider:?}: {}", describe(&err));
            format!("{frontend}/oauth/callback?error=oauth")
        }
    };

    // Ces cookies ne servent qu'une fois : on les efface dans tous les cas.
    let cookies = AppendHeaders([
        (SET_COOKIE, cookie(&config, STATE_COOKIE, "", 0)),
        (SET_COOKIE, cookie(&config, RETURN_COOKIE, "", 0)),
    ]);
    (cookies, Redirect::to(&target)).into_response()
}

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

async fn find_or_create_user(pool: &PgPool, provider: Provider, profile: &OAuthProfile)
    -> Result<i64, sqlx::Error>
{
    // 1. Identite externe deja connue : c'est le cas normal d'une reconnexion.
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

    // 2. Un compte existe deja avec cet email : on y rattache l'identite externe.
    //    Acceptable uniquement parce que l'email est verifie par le fournisseur.
    let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE lower(email) = lower($1)")
        .bind(&profile.email)
        .fetch_optional(&mut *tx)
        .await?;

    // 3. Sinon, nouveau compte sans mot de passe.
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

async fn create_user(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, profile: &OAuthProfile)
    -> Result<i64, sqlx::Error>
{
    let base = sanitize_username(&profile.username_hint);

    // Le pseudo propose peut etre pris : on retente avec un suffixe aleatoire.
    // ON CONFLICT DO NOTHING plutot qu'une erreur, car une erreur SQL
    // annulerait toute la transaction en cours.
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

// Le nom vient du fournisseur, donc de l'exterieur : on ne garde que des
// caracteres surs, et on respecte la limite VARCHAR(32) de la colonne.
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

// HttpOnly : illisible en JavaScript. SameSite=Lax : envoye lors du retour
// depuis le fournisseur (navigation), mais pas sur les requetes d'un autre site.
fn cookie(config: &OAuthConfig, name: &str, value: &str, max_age: u32)
    -> String
{
    let secure = if config.backend_url.starts_with("https://") { "; Secure" } else { "" };
    // Path=/ (pas /auth/oauth) : le navigateur voit ces routes sous /api/auth/oauth
    // depuis le passage par nginx (prefixe /api), le cookie doit donc couvrir
    // large plutot que de dependre d'un prefixe que ce module ne connait pas.
    format!("{name}={value}; Path=/; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}")
}

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

fn random_state()
    -> String
{
    random_hex(32)
}

fn random_hex(bytes: usize)
    -> String
{
    let mut raw = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut raw);
    hex::encode(raw)
}

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

    fn profile(id: &str, email: &str, hint: &str)
        -> OAuthProfile
    {
        OAuthProfile {
            provider_user_id: id.to_string(),
            email: email.to_string(),
            username_hint: hint.to_string(),
        }
    }

    #[test]
    fn sanitize_username_keeps_only_safe_characters()
    {
        assert_eq!(sanitize_username("Jean Dupont"), "JeanDupont");
        assert_eq!(sanitize_username("<script>alert(1)</script>"), "scriptalert1script");
        assert_eq!(sanitize_username("été"), "t");
        assert_eq!(sanitize_username("!!!"), "user");
        assert_eq!(sanitize_username(&"a".repeat(100)).len(), MAX_USERNAME_BASE_LEN);
    }

    #[test]
    fn read_cookie_finds_the_right_cookie()
    {
        let mut headers = HeaderMap::new();
        assert_eq!(read_cookie(&headers, STATE_COOKIE), None);

        headers.insert(COOKIE, "theme=dark; oauth_state=abc123; other=1".parse().unwrap());
        assert_eq!(read_cookie(&headers, STATE_COOKIE), Some("abc123".to_string()));

        // Un cookie vide (apres effacement) ne doit jamais valider un state vide.
        headers.insert(COOKIE, "oauth_state=".parse().unwrap());
        assert_eq!(read_cookie(&headers, STATE_COOKIE), None);
    }

    // Test d'integration : a besoin de la base (make dev).
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

        // Premiere connexion : le compte est cree, sans mot de passe.
        let alice = profile("g-1", "alice@oauth-test.invalid", "Oauth Test Alice");
        let id = find_or_create_user(&pool, Provider::Google, &alice).await.unwrap();
        let row: (String, Option<String>) =
            sqlx::query_as("SELECT username, password_hash FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row, ("OauthTestAlice".to_string(), None));

        // Reconnexion : meme compte, meme si l'email a change chez le fournisseur.
        let renamed = profile("g-1", "other@oauth-test.invalid", "Whatever");
        assert_eq!(find_or_create_user(&pool, Provider::Google, &renamed).await.unwrap(), id);

        // Autre fournisseur, meme email (casse differente) : rattache au meme compte.
        let via_github = profile("gh-9", "ALICE@oauth-test.invalid", "alice-gh");
        assert_eq!(find_or_create_user(&pool, Provider::Github, &via_github).await.unwrap(), id);

        // Autre personne, meme pseudo propose : nouveau compte avec un suffixe.
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
