// ROADMAP — prochaines etapes du backend (voir idea.txt / roadmap.txt)
//
// Phase 1 — Fondations (en cours)
//   [x] Serveur Axum + pool Postgres (switch depuis Actix-web)
//   [x] Structure modulaire (config, db, routes, server)
//   [x] Migrations SQL (table users, sessions, friends, rooms, messages, videos)
//   [x] mod auth : signup/login/logout, hash+salt (argon2), sessions en DB
//   [x] mod errors : enum ApiError + impl IntoResponse (reponses d'erreur propres)
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
//   [ ] Restreindre le CORS aux origines connues (actuellement allow_origin(Any))
//   [ ] Tests multi-users concurrents, README complet

mod auth;
mod config;
mod db;
mod errors;
mod routes;
mod server;

#[tokio::main]
async fn main()
    -> std::io::Result<()>
{
    server::run().await
}
