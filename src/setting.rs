use crate::engine::storage::types::StorageConfig;
use config::{Config, File, FileFormat, Source};
use fs_extra::error;
use serde::Deserialize;
use tracing::error;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct ServiceConfig {
    #[validate(length(min = 1))]
    pub host: String,
    pub http_port: u16,
    pub grpc_port: Option<u16>, // None means that gRPC is disabled
    pub max_request_size_mb: usize,
    pub max_workers: Option<usize>,
    #[serde(default = "default_cors")]
    pub enable_cors: bool,
    #[serde(default)]
    pub enable_tls: bool,
    #[serde(default)]
    pub verify_https_client_certificate: bool,
    pub api_key: Option<String>,
    pub read_only_api_key: Option<String>,

    /// Directory where static files are served from.
    /// For example, the Web-UI should be placed here.
    #[serde(default)]
    pub static_content_dir: Option<String>,

    /// If serving of the static content is enabled.
    /// This includes the Web-UI. True by default.
    #[serde(default)]
    pub enable_static_content: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct Settings {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[validate]
    pub storage: StorageConfig,
    #[validate]
    pub service: ServiceConfig,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_cors() -> bool {
    true
}
const DEFAULT_CONFIG: &str = include_str!("../config/config.yaml");

impl Settings {
    pub fn new(path: Option<String>) -> Result<Self, config::ConfigError> {
        let config_exists = |path| File::with_name(path).collect().is_ok();
        if let Some(path) = path {
            if !config_exists(&path) {
                error!("Config file not found: {}", path);
            }
        }
        let config = Config::builder()
            // Start with compile-time base config
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Yaml));
        let setting: Settings = config.build()?.try_deserialize()?;
        Ok(setting)
    }
}
