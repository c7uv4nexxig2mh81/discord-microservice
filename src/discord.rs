// src/discord.rs
use serde::Deserialize;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::env;
use urlencoding::encode;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

pub fn build_oauth_url(state: &str) -> String {
    format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify&state={}",
        env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set"),
        encode(&env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set")),
        state
    )
}

pub async fn exchange_code(code: &str) -> Option<String> {
    let client_id = env::var("DISCORD_CLIENT_ID").ok()?;
    let client_secret = env::var("DISCORD_CLIENT_SECRET").ok()?;
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").ok()?;

    #[derive(Deserialize)]
    struct T { access_token: String }

    Some(
        CLIENT.post("https://discord.com/api/oauth2/token")
            .form(&[
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &redirect_uri),
                ("scope", "identify")
            ])
            .send().await.ok()?
            .json::<T>().await.ok()?
            .access_token
    )
}

pub async fn get_user_info(token: &str) -> Option<DiscordUser> {
    Some(
        CLIENT.get("https://discord.com/api/users/@me")
            .bearer_auth(token)
            .send().await.ok()?
            .json::<DiscordUser>().await.ok()?
    )
}
