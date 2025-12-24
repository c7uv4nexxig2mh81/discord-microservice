// src/routes.rs
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{discord, session};

#[get("/discord")]
pub async fn discord_oauth() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", discord::build_oauth_url(&Uuid::new_v4().to_string())))
        .finish()
}

#[get("/discord/callback")]
pub async fn discord_callback(query: web::Query<HashMap<String, String>>, req: HttpRequest) -> impl Responder {
    let code = match query.get("code") {
        Some(c) => c,
        None => return HttpResponse::BadRequest().body("Missing code"),
    };

    let user_id = match req.cookie("session").and_then(|c| Some(c.value().to_string()))
        .and_then(|c| futures::executor::block_on(session::verify_session(&c)))
    {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Invalid session"),
    };

    let token = match discord::exchange_code(code).await {
        Some(t) => t,
        None => return HttpResponse::InternalServerError().body("Failed to get Discord token"),
    };

    let discord_user = match discord::get_user_info(&token).await {
        Some(u) => u,
        None => return HttpResponse::InternalServerError().body("Failed to fetch Discord user"),
    };

    let discord_tag = format!("{}#{}", discord_user.username, discord_user.discriminator);

    if !session::link_discord(&user_id, &discord_user.id, &discord_tag).await {
        return HttpResponse::InternalServerError().body("Failed to link Discord account");
    }

    HttpResponse::Ok().json(json!({
        "user_id": user_id,
        "discord_id": discord_user.id,
        "discord_username": discord_tag
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(discord_oauth);
    cfg.service(discord_callback);
}
