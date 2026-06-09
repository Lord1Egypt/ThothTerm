use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {name}")]
    NotFound { name: String },

    #[error("Plugin manifest missing or invalid at {path}: {reason}")]
    InvalidManifest { path: PathBuf, reason: String },

    #[error("Plugin WASM file not found at {path}")]
    WasmNotFound { path: PathBuf },

    #[error("WASM compilation failed for {plugin}: {reason}")]
    CompilationFailed { plugin: String, reason: String },

    #[error("WASM instantiation failed for {plugin}: {reason}")]
    InstantiationFailed { plugin: String, reason: String },

    #[error("Plugin API version mismatch: plugin requires {required}, host provides {provided}")]
    ApiVersionMismatch { required: String, provided: String },

    #[error("Permission denied: plugin {plugin} requires {permission}")]
    PermissionDenied { plugin: String, permission: String },

    #[error("Plugin execution panicked: {plugin} — {message}")]
    ExecutionPanic { plugin: String, message: String },

    #[error("Plugin download failed from {url}: {reason}")]
    DownloadFailed { url: String, reason: String },

    #[error("Plugin directory error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Plugin manifest parse error: {0}")]
    ManifestParse(#[from] toml::de::Error),

    #[error("Plugin {name} is already installed")]
    AlreadyInstalled { name: String },

    #[error("Plugin {name} is not enabled")]
    NotEnabled { name: String },

    #[error("WASM file not found: {path}")]
    WasmFile { path: std::path::PathBuf },

    #[error("WASM load/compile failed: {reason}")]
    WasmLoad { reason: String },

    #[error("Hook '{hook}' execution failed: {reason}")]
    HookFailed { hook: String, reason: String },
}

pub type PluginResult<T> = Result<T, PluginError>;
