// src/session.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use once_cell::sync::Lazy;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

#[derive(Debug, Deserialize)]
struct SessionResponse { user_id: String }

pub async fn verify_session(cookie: &str) -> Option<String> {
    if cookie.is_empty() { return None; }
    let url = format!("{}/internal/auth/session", env::var("MAIN_API_URL").ok()?);

    Some(
        CLIENT.get(&url)
            .header("Cookie", format!("session={}", cookie))
            .send().await.ok()?
            .error_for_status().ok()?
            .json::<SessionResponse>().await.ok()?
            .user_id
    )
}

#[derive(Serialize)]
struct LinkPayload<'a> { discord_id: &'a str, discord_username: &'a str }

pub async fn link_discord(user_id: &str, discord_id: &str, discord_username: &str) -> bool {
    let url = match env::var("MAIN_API_URL") {
        Ok(u) => format!("{}/internal/user/discord-link/{}", u, user_id),
        Err(_) => return false,
    };

    CLIENT.post(&url)
        .json(&LinkPayload { discord_id, discord_username })
        .send().await.map_or(false, |r| r.status().is_success())
}
