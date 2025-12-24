use actix_web::{App, HttpServer};
mod config;
mod routes;
mod discord;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize configuration and logger
    let _cfg = config::init();

    log::info!("listening OAuth microservice on 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .configure(routes::init) // Mount all routes
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
