use serde::Deserialize;
use std::env;
use reqwest::Client;

/// Discord user information
#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    // avatar field removed to silence warning
}

/// Build Discord OAuth2 URL for user authorization
pub fn build_oauth_url(state: &str) -> String {
    let client_id = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set");
    let scope = "identify";

    format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        scope,
        state
    )
}

/// Exchange Discord OAuth2 code for access token
pub async fn exchange_code(code: &str) -> Option<String> {
    let client_id = env::var("DISCORD_CLIENT_ID").ok()?;
    let client_secret = env::var("DISCORD_CLIENT_SECRET").ok()?;
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").ok()?;

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri.as_str()),
        ("scope", "identify"),
    ];

    let client = Client::new();
    let resp = client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await
        .ok()?;

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let token: TokenResponse = resp.json().await.ok()?;
    Some(token.access_token)
}

/// Fetch Discord user info using access token
pub async fn get_user_info(access_token: &str) -> Option<DiscordUser> {
    let client = Client::new();
    let resp = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .ok()?;

    let user: DiscordUser = resp.json().await.ok()?;
    Some(user)
}
