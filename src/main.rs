// Entry point of the backend binary. Only declares the application's
// modules and starts the server: all the actual logic lives in `server`
// and the business modules (auth, rooms, ws, media, ...).

mod auth;
mod config;
mod db;
mod errors;
mod friends;
mod media;
mod notifications;
mod recommendations;
mod rooms;
mod routes;
mod server;
mod users;
mod ws;
mod health;

// Starts the async runtime and immediately delegates to `server::run`.
#[tokio::main]
async fn main() -> std::io::Result<()>
{
    server::run().await
}
