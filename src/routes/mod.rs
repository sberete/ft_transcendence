use actix_web::web;

// Chaque phase ajoute son propre sous-module ici (auth, users, friends, ws, media...)
// et l'enregistre dans configure().
pub fn configure(_cfg: &mut web::ServiceConfig) {}
