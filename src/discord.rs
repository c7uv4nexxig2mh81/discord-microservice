// src/discord.rs
use serde::Deserialize;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::env;
use urlencoding::encode;

/// Shared HTTP client for all Discord requests
static CLIENT: Lazy<Client> = Lazy::new(Client::new);

/// Discord user information
#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

/// Build Discord OAuth2 authorization URL
pub fn build_oauth_url(state: &str) -> String {
    let client_id = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set");
    format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify&state={}",
        client_id,
        encode(&redirect_uri),
        state
    )
}

/// Exchange OAuth2 code for access token
pub async fn exchange_code(code: &str) -> Option<String> {
    let client_id = env::var("DISCORD_CLIENT_ID").ok()?;
    let client_secret = env::var("DISCORD_CLIENT_SECRET").ok()?;
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").ok()?;

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    CLIENT
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("scope", "identify"),
        ])
        .send()
        .await
        .ok()?
        .json::<TokenResponse>()
        .await
        .ok()
        .map(|t| t.access_token)
}

/// Fetch Discord user info using an access token
pub async fn get_user_info(access_token: &str) -> Option<DiscordUser> {
    CLIENT
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .ok()?
        .json::<DiscordUser>()
        .await
        .ok()
}
