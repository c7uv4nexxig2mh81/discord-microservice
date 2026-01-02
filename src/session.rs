// src/session.rs

use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

const AUTH_SESSION_PATH: &str = "/internal/auth/session";
const DISCORD_LINK_PATH: &str = "/internal/user/discord-link";

/// Session verification response
#[derive(Debug, Deserialize)]
struct SessionResponse {
    user_id: String,
}

/// Session-related errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Missing or empty session cookie")]
    MissingSession,

    #[error("Session verification failed")]
    VerificationFailed,

    #[error("Unexpected response status: {0}")]
    BadStatus(StatusCode),

    #[error("Failed to parse response")]
    ParseError,
}

/// Verify user session via main API
pub async fn verify_session(
    config: &Config,
    session_cookie: &str,
) -> Result<String, SessionError> {
    if session_cookie.is_empty() {
        return Err(SessionError::MissingSession);
    }

    let url = format!("{}{}", config.main_api_url, AUTH_SESSION_PATH);

    let res = CLIENT
        .get(url)
        .header("Cookie", format!("session={}", session_cookie))
        .send()
        .await
        .map_err(|_| SessionError::VerificationFailed)?;

    if !res.status().is_success() {
        return Err(SessionError::BadStatus(res.status()));
    }

    res.json::<SessionResponse>()
        .await
        .map(|r| r.user_id)
        .map_err(|_| SessionError::ParseError)
}

/// Discord link payload
#[derive(Debug, Serialize)]
struct LinkPayload<'a> {
    discord_id: &'a str,
    discord_username: &'a str,
}

/// Link Discord account to user
pub async fn link_discord(
    config: &Config,
    user_id: &str,
    discord_id: &str,
    discord_username: &str,
) -> Result<(), SessionError> {
    let url = format!(
        "{}/{}{}",
        config.main_api_url,
        DISCORD_LINK_PATH,
        format!("/{}", user_id)
    );

    let res = CLIENT
        .post(url)
        .json(&LinkPayload {
            discord_id,
            discord_username,
        })
        .send()
        .await
        .map_err(|_| SessionError::VerificationFailed)?;

    if !res.status().is_success() {
        return Err(SessionError::BadStatus(res.status()));
    }

    Ok(())
}
