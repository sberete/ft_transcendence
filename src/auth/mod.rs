// Authentication module: signup/login/logout with hashed passwords,
// session validation, OAuth 2.0 (Google/GitHub), and login rate
// limiting. Re-exports the pieces other modules need (the `AuthUser`
// extractor, session validation) so they don't reach into submodules.

mod extractor;
mod handlers;
mod oauth;
mod password;
pub mod rate_limit;
mod session;

use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

pub use extractor::AuthUser;
pub use oauth::OAuthConfig;
pub use rate_limit::LoginRateLimiter;
pub use session::validate_session;

// REST routes for this module: local auth plus the OAuth flow.
pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/signup", post(handlers::signup))
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/oauth/providers", get(oauth::providers))
        .route("/oauth/{provider}/start", get(oauth::start))
        .route("/oauth/{provider}/callback", get(oauth::callback))
}
