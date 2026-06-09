use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found at {path}")]
    NotFound { path: PathBuf },

    #[error("Failed to read config file {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse TOML config: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse JSON config: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Failed to serialize config to TOML: {0}")]
    TomlSerialize(String),

    #[error("Failed to write config: {0}")]
    WriteFailed(#[from] std::io::Error),

    #[error("Invalid font size {size}: must be between 6.0 and 72.0")]
    InvalidFontSize { size: f32 },

    #[error("Invalid opacity {opacity}: must be between 0.0 and 1.0")]
    InvalidOpacity { opacity: f32 },

    #[error("Unknown color scheme: {name}")]
    UnknownColorScheme { name: String },

    #[error("Config directory could not be determined")]
    NoConfigDir,
}

pub type ConfigResult<T> = Result<T, ConfigError>;
