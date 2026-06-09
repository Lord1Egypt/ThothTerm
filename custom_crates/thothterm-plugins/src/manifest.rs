use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const HOST_API_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry: String,
    pub api_version: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub hooks: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl PluginManifest {
    pub fn load(dir: &Path) -> PluginResult<Self> {
        let path = dir.join("plugin.toml");

        if !path.exists() {
            return Err(PluginError::InvalidManifest {
                path: path.clone(),
                reason: "plugin.toml not found".into(),
            });
        }

        let content = std::fs::read_to_string(&path)?;
        let manifest: Self = toml::from_str(&content)?;
        manifest.validate(dir)?;
        Ok(manifest)
    }

    pub fn validate(&self, dir: &Path) -> PluginResult<()> {
        // Check API version compatibility
        if !is_compatible_api_version(&self.plugin.api_version, HOST_API_VERSION) {
            return Err(PluginError::ApiVersionMismatch {
                required: self.plugin.api_version.clone(),
                provided: HOST_API_VERSION.into(),
            });
        }

        // Check WASM entry file exists
        let wasm_path = dir.join(&self.plugin.entry);
        if !wasm_path.exists() {
            return Err(PluginError::WasmNotFound { path: wasm_path });
        }

        // Validate name — no spaces, no special chars
        if self.plugin.name.contains(char::is_whitespace) || self.plugin.name.is_empty() {
            return Err(PluginError::InvalidManifest {
                path: dir.join("plugin.toml"),
                reason: "Plugin name must not be empty or contain whitespace".into(),
            });
        }

        Ok(())
    }

    pub fn wasm_path(&self, install_dir: &Path) -> PathBuf {
        install_dir.join(&self.plugin.name).join(&self.plugin.entry)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.plugin.permissions.iter().any(|p| p == permission)
    }

    pub fn has_hook(&self, hook: &str) -> bool {
        self.plugin.hooks.iter().any(|h| h == hook)
    }
}

/// Check if a plugin's required API version is compatible with host version.
/// Simple semver major version check: major must match, minor can be >=.
fn is_compatible_api_version(required: &str, host: &str) -> bool {
    let req_parts: Vec<u32> = required.split('.').filter_map(|s| s.parse().ok()).collect();
    let host_parts: Vec<u32> = host.split('.').filter_map(|s| s.parse().ok()).collect();

    match (req_parts.first(), host_parts.first()) {
        (Some(req_major), Some(host_major)) => req_major == host_major,
        _ => false,
    }
}
