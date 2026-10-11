pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub peertube_url: String,
    // URL publique du backend : sert a construire le redirect_uri OAuth.
    pub backend_url: String,
    // Adresses du front autorisees apres une connexion OAuth, separees par des
    // virgules. La premiere sert par defaut.
    pub frontend_urls: String,
    // Optionnels : un fournisseur sans identifiants est simplement desactive.
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
}

impl Config {
    pub fn init()
        -> Self
    {
        dotenvy::dotenv().ok();
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

        Self::from_env()
    }

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
        // Par defaut : Vite (npm run dev) et nginx (make up).
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

// Une variable vide ("GOOGLE_CLIENT_ID=") compte comme absente.
fn optional_env(name: &str)
    -> Option<String>
{
    std::env::var(name).ok().filter(|value| !value.trim().is_empty())
}
