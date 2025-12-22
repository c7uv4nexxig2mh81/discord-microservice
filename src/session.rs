use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use once_cell::sync::Lazy;

/// Shared static HTTP client for reuse
static CLIENT: Lazy<Client> = Lazy::new(Client::new);

/// Response from main API session verification
#[derive(Debug, Deserialize)]
struct SessionResponse {
    user_id: String,
}

/// Verify a session cookie via the main API.
/// Returns `Some(user_id)` if valid, `None` otherwise.
pub async fn verify_session(session_cookie: &str) -> Option<String> {
    if session_cookie.is_empty() {
        return None;
    }

    let main_api_url = env::var("MAIN_API_URL").ok()?;
    let url = format!("{}/internal/auth/session", main_api_url);

    CLIENT
        .get(&url)
        .header("Cookie", format!("session={}", session_cookie))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<SessionResponse>()
        .await
        .ok()
        .map(|s| s.user_id)
}

/// Payload structure for linking Discord accounts
#[derive(Serialize)]
struct LinkDiscordPayload<'a> {
    discord_id: &'a str,
    discord_username: &'a str,
}

/// Link a Discord account to a user via the main API.
/// Returns `true` if successful, `false` otherwise.
pub async fn link_discord(user_id: &str, discord_id: &str, discord_username: &str) -> bool {
    let main_api_url = match env::var("MAIN_API_URL") {
        Ok(url) => url,
        Err(_) => return false,
    };

    let payload = LinkDiscordPayload {
        discord_id,
        discord_username,
    };

    let url = format!("{}/internal/user/discord-link/{}", main_api_url, user_id);

    CLIENT
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_or(false, |resp| resp.status().is_success())
}
