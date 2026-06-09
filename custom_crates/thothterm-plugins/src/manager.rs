use crate::{
    error::{PluginError, PluginResult},
    manifest::PluginManifest,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Manages plugin installation, loading, and lifecycle.
pub struct PluginManager {
    /// Root directory where plugins are installed.
    plugins_dir: PathBuf,
    /// Loaded and enabled plugins.
    loaded: HashMap<String, LoadedPlugin>,
}

struct LoadedPlugin {
    manifest: PluginManifest,
    enabled: bool,
}

impl PluginManager {
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            loaded: HashMap::new(),
        }
    }

    /// Default plugin directory, respects portable mode via thothterm-config RunMode.
    pub fn default_dir() -> PathBuf {
        thothterm_config::RunMode::detect().plugins_dir()
    }

    /// Install a plugin from a local directory.
    pub fn install_local(&mut self, plugin_dir: &Path) -> PluginResult<String> {
        let manifest = PluginManifest::load(plugin_dir)?;
        let name = manifest.plugin.name.clone();

        let dest = self.plugins_dir.join(&name);
        if dest.exists() {
            return Err(PluginError::AlreadyInstalled { name });
        }

        std::fs::create_dir_all(&dest)?;
        copy_dir_contents(plugin_dir, &dest)?;

        info!("Installed plugin: {}", name);
        self.loaded.insert(
            name.clone(),
            LoadedPlugin {
                manifest,
                enabled: true,
            },
        );

        Ok(name)
    }

    /// Load all installed plugins from the plugins directory.
    pub fn load_all(&mut self) -> Vec<PluginResult<String>> {
        if !self.plugins_dir.exists() {
            return Vec::new();
        }

        let entries = match std::fs::read_dir(&self.plugins_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("Cannot read plugins dir: {}", e);
                return Vec::new();
            }
        };

        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if !path.is_dir() {
                    return None;
                }
                Some(self.load_from_dir(&path))
            })
            .collect()
    }

    fn load_from_dir(&mut self, dir: &Path) -> PluginResult<String> {
        let manifest = PluginManifest::load(dir)?;
        let name = manifest.plugin.name.clone();

        debug!("Loading plugin: {}", name);
        self.loaded.insert(
            name.clone(),
            LoadedPlugin {
                manifest,
                enabled: true,
            },
        );

        Ok(name)
    }

    /// Enable a plugin.
    pub fn enable(&mut self, name: &str) -> PluginResult<()> {
        let plugin = self.loaded.get_mut(name).ok_or_else(|| PluginError::NotFound {
            name: name.to_string(),
        })?;
        plugin.enabled = true;
        info!("Enabled plugin: {}", name);
        Ok(())
    }

    /// Disable a plugin.
    pub fn disable(&mut self, name: &str) -> PluginResult<()> {
        let plugin = self.loaded.get_mut(name).ok_or_else(|| PluginError::NotFound {
            name: name.to_string(),
        })?;
        plugin.enabled = false;
        info!("Disabled plugin: {}", name);
        Ok(())
    }

    /// Remove a plugin completely.
    pub fn remove(&mut self, name: &str) -> PluginResult<()> {
        if !self.loaded.contains_key(name) {
            return Err(PluginError::NotFound {
                name: name.to_string(),
            });
        }

        let plugin_dir = self.plugins_dir.join(name);
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)?;
        }

        self.loaded.remove(name);
        info!("Removed plugin: {}", name);
        Ok(())
    }

    /// List all loaded plugins.
    pub fn list(&self) -> Vec<PluginListEntry> {
        self.loaded
            .values()
            .map(|p| PluginListEntry {
                name: p.manifest.plugin.name.clone(),
                version: p.manifest.plugin.version.clone(),
                description: p.manifest.plugin.description.clone(),
                enabled: p.enabled,
            })
            .collect()
    }

    /// Check if a plugin is loaded and enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.loaded
            .get(name)
            .map(|p| p.enabled)
            .unwrap_or(false)
    }

    pub fn plugin_count(&self) -> usize {
        self.loaded.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.loaded.values().filter(|p| p.enabled).count()
    }

    /// Fire a hook on all enabled plugins that declare it.
    /// Silently skips plugins that don't have the hook.
    #[cfg(feature = "wasm")]
    pub fn run_hook(&self, hook_name: &str) {
        use crate::engine::WasmEngine;
        let engine = match WasmEngine::new() {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to create WASM engine: {}", e);
                return;
            }
        };

        for (name, plugin) in &self.loaded {
            if !plugin.enabled {
                continue;
            }
            if !plugin.manifest.has_hook(hook_name) {
                continue;
            }
            let wasm_path = self.plugins_dir.join(name).join(&plugin.manifest.plugin.entry);
            debug!("Running hook '{}' for plugin '{}'", hook_name, name);
            if let Err(e) = engine.run_hook(&wasm_path, hook_name) {
                warn!("Hook '{}' failed for plugin '{}': {}", hook_name, name, e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginListEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

fn copy_dir_contents(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&dest)?;
            copy_dir_contents(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
