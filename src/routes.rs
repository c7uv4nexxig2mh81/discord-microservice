// src/routes.rs

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{config, discord, session};

/// Typed OAuth callback query
#[derive(Debug, Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    state: String,
}

/// Redirect user to Discord OAuth
#[get("/discord")]
pub async fn discord_oauth() -> impl Responder {
    let config = config::init();

    // Generate CSRF-safe OAuth state
    let state = Uuid::new_v4().to_string();

    HttpResponse::Found()
        .append_header((
            "Location",
            discord::build_oauth_url(config, &state),
        ))
        .finish()
}

/// Discord OAuth callback
#[get("/discord/callback")]
pub async fn discord_callback(
    query: web::Query<OAuthCallbackQuery>,
    req: HttpRequest,
) -> impl Responder {
    let config = config::init();

    // Validate session cookie
    let session_cookie = match req.cookie("session") {
        Some(c) => c.value().to_owned(),
        None => {
            return HttpResponse::Unauthorized().body("Missing session cookie");
        }
    };

    let user_id = match session::verify_session(&session_cookie).await {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().body("Invalid session");
        }
    };

    // Exchange OAuth code
    let token = match discord::exchange_code(config, &query.code).await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Discord token exchange failed: {}", e);
            return HttpResponse::BadGateway().body("Discord OAuth failed");
        }
    };

    // Fetch Discord user
    let discord_user = match discord::get_user_info(&token).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Discord user fetch failed: {}", e);
            return HttpResponse::BadGateway().body("Failed to fetch Discord user");
        }
    };

    let discord_tag = format!(
        "{}#{}",
        discord_user.username,
        discord_user.discriminator
    );

    // Link Discord account
    if !session::link_discord(
        &user_id,
        &discord_user.id,
        &discord_tag,
    )
    .await
    {
        log::error!("Failed to link Discord account for user {}", user_id);
        return HttpResponse::InternalServerError()
            .body("Failed to link Discord account");
    }

    HttpResponse::Ok().json(json!({
        "user_id": user_id,
        "discord_id": discord_user.id,
        "discord_username": discord_tag
    }))
}

/// Register routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(discord_oauth);
    cfg.service(discord_callback);
}
