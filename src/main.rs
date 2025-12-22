// src/main.rs
use actix_web::{App, HttpServer};
mod config;
mod routes;
mod discord;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration and initialize logger
    let _cfg = config::init();

    // Log startup info
    log::info!("Starting Discord OAuth microservice on 0.0.0.0:8080");

    // Start HTTP server
    HttpServer::new(|| {
        App::new()
            .configure(routes::init) // Mount all routes
    })
    .bind("0.0.0.0:8080")? // Listen on port 8080
    .run()
    .await
}
