use actix_web::{web, get, HttpResponse, Responder, HttpRequest};
use uuid::Uuid;
use std::collections::HashMap;

use crate::{discord, session};

/// Redirect user to Discord OAuth2 authorization page
#[get("/discord")]
pub async fn discord_oauth() -> impl Responder {
    let state = Uuid::new_v4().to_string(); // CSRF protection
    let url = discord::build_oauth_url(&state);

    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}

/// Handle Discord OAuth2 callback and link account
#[get("/discord/callback")]
pub async fn discord_callback(
    query: web::Query<HashMap<String, String>>,
    req: HttpRequest,
) -> impl Responder {
    let code = match query.get("code") {
        Some(c) => c,
        None => return HttpResponse::BadRequest().body("Missing code"),
    };

    let session_cookie = match req.cookie("session") {
        Some(c) => c.value().to_string(),
        None => return HttpResponse::Unauthorized().body("Missing session cookie"),
    };

    // Verify session via main API
    let user_id = match session::verify_session(&session_cookie).await {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Invalid session"),
    };

    // Exchange code for Discord access token
    let token = match discord::exchange_code(code).await {
        Some(t) => t,
        None => return HttpResponse::InternalServerError().body("Failed to get Discord token"),
    };

    // Fetch Discord user info
    let discord_user = match discord::get_user_info(&token).await {
        Some(u) => u,
        None => return HttpResponse::InternalServerError().body("Failed to fetch Discord user"),
    };

    let discord_tag = format!("{}#{}", discord_user.username, discord_user.discriminator);

    // Link Discord account via main API
    if !session::link_discord(&user_id, &discord_user.id, &discord_tag).await {
        return HttpResponse::InternalServerError().body("Failed to link Discord account");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "user_id": user_id,
        "discord_id": discord_user.id,
        "discord_username": discord_tag
    }))
}

/// Mount routes for Actix
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(discord_oauth);
    cfg.service(discord_callback);
}
