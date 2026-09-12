// ROADMAP — prochaines etapes du backend (voir idea.txt / roadmap.txt)
//
// Phase 1 — Fondations (en cours)
//   [x] Serveur Actix-web + pool Postgres + /health
//   [x] Structure modulaire (config, db, routes)
//   [ ] Migrations SQL (table users, sessions, friends, rooms, messages, videos)
//   [ ] mod auth : signup/login, hash+salt (argon2), sessions/JWT
//   [ ] mod errors : enum ApiError + impl ResponseError (reponses d'erreur propres)
//   [ ] Definir le contrat d'API (routes REST + evenements WebSocket) pour le(s) frontend(s)
//
// Phase 2 — Utilisateurs & social
//   [ ] mod users : CRUD profil, upload avatar
//   [ ] mod friends : ajout/suppression, statut en ligne
//   [ ] OAuth (Google/GitHub)
//
// Phase 3 — Coeur temps reel (cf. day07 de la piscine Rust)
//   [ ] mod ws : infra WebSocket (connexions, rooms, presence)
//   [ ] Chat live
//   [ ] Sync de lecture (play/pause/seek) par room
//
// Phase 4 — Contenu & IA
//   [ ] mod media : upload/streaming video (HTTP Range requests)
//   [ ] mod recommend : recommandation films/series
//   [ ] Notifications, recherche avancee
//
// Phase 5 — Durcissement
//   [ ] Module de securite restant (2FA ou remplacant), ORM finalise
//   [ ] HTTPS, Privacy Policy / ToS, nettoyage warnings
//   [ ] Tests multi-users concurrents, README complet

mod config;
mod db;
mod routes;

use actix_web::{web, App, HttpServer};
use config::Config;

async fn run() -> std::io::Result<()> {
    let config = Config::init();

    let pool = db::init_pool(&config.database_url)
        .await
        .expect("failed to connect to the database");

    log::info!("Starting server on 0.0.0.0:{}", config.port);

    let port = config.port;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
}
