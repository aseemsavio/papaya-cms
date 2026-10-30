use std::sync::LazyLock;

use config::{Config as ConfigBuilder, Environment, File};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub s3: S3Config,
    pub site: SiteConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    /// Folder inside the bucket that holds this site's objects.
    pub root_folder: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub force_path_style: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteConfig {
    pub name: String,
    pub description: String,
    /// Optional description for the home page. Falls back to `description` when empty.
    #[serde(default)]
    pub home_description: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub refresh_token: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Missing required configuration: {0}")]
    Missing(String),
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let environment = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let builder = ConfigBuilder::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name(&format!("config.{}", environment)).required(false))
            .add_source(Environment::default().separator("__"));

        let config: Config = builder.build()?.try_deserialize()?;

        if config.s3.endpoint.is_empty() {
            return Err(ConfigError::Missing("S3 endpoint is required".into()));
        }
        if config.s3.bucket.is_empty() {
            return Err(ConfigError::Missing("S3 bucket is required".into()));
        }
        if config.s3.root_folder.trim_matches('/').is_empty() {
            return Err(ConfigError::Missing("S3 root folder is required".into()));
        }
        if config.s3.access_key.is_empty() {
            return Err(ConfigError::Missing("S3 access key is required".into()));
        }
        if config.s3.secret_key.is_empty() {
            return Err(ConfigError::Missing("S3 secret key is required".into()));
        }

        Ok(config)
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::load().expect("Failed to load configuration"));