mod auth;
mod config;
mod db;
mod errors;
mod friends;
mod media;
mod routes;
mod server;
mod users;
mod ws;

#[tokio::main]
async fn main() -> std::io::Result<()>
{
    server::run().await
}
