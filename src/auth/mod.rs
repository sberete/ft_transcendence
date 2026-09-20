mod handlers;
mod password;
mod session;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) 
{
    cfg.service(
        web::scope("/auth")
            .route("/signup", web::post().to(handlers::signup))
            .route("/login", web::post().to(handlers::login))
            .route("/logout", web::post().to(handlers::logout)),
    );
}
