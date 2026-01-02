// src/discord.rs

use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use urlencoding::encode;

use crate::config::Config;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

const DISCORD_API: &str = "https://discord.com/api";
const OAUTH_AUTHORIZE: &str = "https://discord.com/api/oauth2/authorize";
const OAUTH_TOKEN: &str = "https://discord.com/api/oauth2/token";

/// Discord OAuth user
#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

/// Discord OAuth errors
#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("Discord API request failed")]
    RequestFailed,

    #[error("Unexpected Discord response status: {0}")]
    BadStatus(StatusCode),

    #[error("Failed to parse Discord response")]
    ParseError,
}

/// Build Discord OAuth URL
pub fn build_oauth_url(config: &Config, state: &str) -> String {
    format!(
        "{url}?client_id={client_id}&redirect_uri={redirect}&response_type=code&scope=identify&state={state}",
        url = OAUTH_AUTHORIZE,
        client_id = config.discord_client_id,
        redirect = encode(&config.discord_redirect_uri),
        state = encode(state),
    )
}

/// Exchange OAuth code for access token
pub async fn exchange_code(
    config: &Config,
    code: &str,
) -> Result<String, DiscordError> {
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let res = CLIENT
        .post(OAUTH_TOKEN)
        .form(&[
            ("client_id", config.discord_client_id.as_str()),
            ("client_secret", config.discord_client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", config.discord_redirect_uri.as_str()),
            ("scope", "identify"),
        ])
        .send()
        .await
        .map_err(|_| DiscordError::RequestFailed)?;

    if !res.status().is_success() {
        return Err(DiscordError::BadStatus(res.status()));
    }

    res.json::<TokenResponse>()
        .await
        .map(|t| t.access_token)
        .map_err(|_| DiscordError::ParseError)
}

/// Fetch Discord user info
pub async fn get_user_info(token: &str) -> Result<DiscordUser, DiscordError> {
    let res = CLIENT
        .get(format!("{}/users/@me", DISCORD_API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| DiscordError::RequestFailed)?;

    if !res.status().is_success() {
        return Err(DiscordError::BadStatus(res.status()));
    }

    res.json::<DiscordUser>()
        .await
        .map_err(|_| DiscordError::ParseError)
}
