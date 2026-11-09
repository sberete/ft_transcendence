// Global backend configuration, loaded once at startup from environment
// variables (and a .env file in dev). Every other module reads this
// struct instead of calling `env::var` itself, to keep a single source
// of truth for configuration.

pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub peertube_url: String,
    // Public URL of the backend: used to build the OAuth redirect_uri.
    pub backend_url: String,
    // Frontend addresses allowed after an OAuth login, comma-separated.
    // The first one is the default.
    pub frontend_urls: String,
    // Optional: a provider with no credentials is simply disabled.
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
}

impl Config {
    // Entry point called from `server::run`: loads the `.env` file (if any),
    // initializes the logger, then builds the config from the environment.
    pub fn init()
        -> Self
    {
        dotenvy::dotenv().ok();
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

        Self::from_env()
    }

    // Reads each environment variable, with sensible defaults for local
    // development wherever possible.
    fn from_env()
        -> Self
    {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a valid number");
        let peertube_url = std::env::var("PEERTUBE_URL")
            .unwrap_or_else(|_| "https://framatube.org".to_string());

        let backend_url = std::env::var("BACKEND_URL")
            .unwrap_or_else(|_| format!("http://localhost:{port}"));
        // Default: Vite (npm run dev) and nginx (make up).
        let frontend_urls = std::env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:5173,http://localhost:8080".to_string());

        Config {
            database_url,
            port,
            peertube_url,
            backend_url,
            frontend_urls,
            google_client_id: optional_env("GOOGLE_CLIENT_ID"),
            google_client_secret: optional_env("GOOGLE_CLIENT_SECRET"),
            github_client_id: optional_env("GITHUB_CLIENT_ID"),
            github_client_secret: optional_env("GITHUB_CLIENT_SECRET"),
        }
    }
}

impl Config {
    // The origins (scheme+host+port, no path) allowed to talk to the
    // backend outside of REST: OAuth redirects, and the Origin header
    // check when a WebSocket connection opens (not covered by CORS).
    pub fn frontend_origins(&self)
        -> Vec<String>
    {
        self.frontend_urls
            .split(',')
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .filter_map(origin_of)
            .collect()
    }
}

// Extracts the origin (scheme+host+port) from a full URL, or `None` if
// the origin is "opaque" (data:, file:...) and so names no actual site.
fn origin_of(url: &str)
    -> Option<String>
{
    let origin = reqwest::Url::parse(url).ok()?.origin();
    origin.is_tuple().then(|| origin.ascii_serialization())
}

// Reads an optional environment variable: missing or empty (e.g.
// "GOOGLE_CLIENT_ID=") are both treated the same way, as disabled.
fn optional_env(name: &str)
    -> Option<String>
{
    std::env::var(name).ok().filter(|value| !value.trim().is_empty())
}
