// src/config.rs

use dotenv::dotenv;
use std::{env, fmt};
use once_cell::sync::OnceCell;

/// Global config instance
static CONFIG: OnceCell<Config> = OnceCell::new();

/// App configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_redirect_uri: String,
    pub main_api_url: String,
    pub log_level: String,
}

/// Config loading error
#[derive(Debug)]
pub enum ConfigError {
    MissingVar(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVar(key) => write!(f, "Missing required environment variable: {}", key),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        Ok(Self {
            discord_client_id: required("DISCORD_CLIENT_ID")?,
            discord_client_secret: required("DISCORD_CLIENT_SECRET")?,
            discord_redirect_uri: required("DISCORD_REDIRECT_URI")?,
            main_api_url: required("MAIN_API_URL")?,
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        })
    }
}

/// Read a required environment variable
fn required(key: &'static str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::MissingVar(key))
}

/// Initialize configuration + logging (safe to call multiple times)
pub fn init() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config = Config::from_env()
            .unwrap_or_else(|e| panic!("Configuration error: {}", e));

        init_logger(&config.log_level);

        log::info!("Configuration loaded");
        config
    })
}

/// Initialize logger once
fn init_logger(level: &str) {
    use env_logger::{Builder, Env};

    let _ = Builder::from_env(Env::default().default_filter_or(level))
        .is_test(cfg!(test))
        .try_init();
}
