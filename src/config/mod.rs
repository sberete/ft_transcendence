pub struct Config {
    pub database_url: String,
    pub port: u16,
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

        Config { database_url, port }
    }
}
