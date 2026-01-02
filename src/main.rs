use actix_web::{App, HttpServer};

mod config;
mod routes;
mod discord;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize configuration + logger once
    let _config = config::init();

    log::info!("OAuth microservice listening on 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .configure(routes::init)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
