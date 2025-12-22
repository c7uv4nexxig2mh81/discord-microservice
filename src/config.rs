// src/config.rs
use dotenv::dotenv;
use std::env;

/// App configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_redirect_uri: String,
    pub main_api_url: String,
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            discord_client_id: get_env("DISCORD_CLIENT_ID"),
            discord_client_secret: get_env("DISCORD_CLIENT_SECRET"),
            discord_redirect_uri: get_env("DISCORD_REDIRECT_URI"),
            main_api_url: get_env("MAIN_API_URL"),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        }
    }
}

/// Helper to read env variable or panic if missing
fn get_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Environment variable {} is not set", key))
}

/// Initialize configuration and logging
pub fn init() -> Config {
    let config = Config::from_env();

    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&config.log_level))
        .init();

    log::info!("Configuration loaded successfully");
    config
}
