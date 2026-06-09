pub mod error;
pub mod run_mode;

pub use run_mode::RunMode;

pub use error::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

// ── Top-level config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThothConfig {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub ai: AiConfig,
    pub web3: Web3Config,
    pub keybindings: KeybindingsConfig,
    pub ssh: SshConfig,
    pub plugins: PluginsConfig,
}

// ── Sections ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub default_shell: String,
    pub scrollback_lines: usize,
    pub startup_wizard: bool,
    pub check_for_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub font_family: String,
    pub font_size: f32,
    pub theme: String,
    pub opacity: f32,
    pub background_image: Option<String>,
    pub cursor_style: CursorStyle,
    pub enable_animations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    pub enabled: bool,
    pub provider: AiProvider,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub suggestions: bool,
    pub error_analysis: bool,
    pub privacy_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Ollama,
    OpenAi,
    Claude,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Web3Config {
    pub enabled: bool,
    pub rpc_url: String,
    pub gas_tracker: bool,
    pub ens_resolver: bool,
    pub auto_detect_projects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    pub new_tab: String,
    pub close_tab: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub command_palette: String,
    pub ai_panel: String,
    pub settings: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SshConfig {
    pub profiles: Vec<SshProfile>,
    pub keepalive_interval: u32,
    pub keepalive_max_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_path: Option<String>,
    pub jump_host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    pub auto_update: bool,
    pub plugins: Vec<InstalledPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub name: String,
    pub source: String,
    pub version: String,
    pub enabled: bool,
}

// ── Defaults ──────────────────────────────────────────────────────────────────

impl Default for ThothConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            ai: AiConfig::default(),
            web3: Web3Config::default(),
            keybindings: KeybindingsConfig::default(),
            ssh: SshConfig::default(),
            plugins: PluginsConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_shell: detect_default_shell(),
            scrollback_lines: 10_000,
            startup_wizard: true,
            check_for_updates: true,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".into(),
            font_size: 14.0,
            theme: "catppuccin-mocha".into(),
            opacity: 1.0,
            background_image: None,
            cursor_style: CursorStyle::Block,
            enable_animations: true,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AiProvider::Ollama,
            model: "llama3.2".into(),
            base_url: "http://localhost:11434".into(),
            api_key: String::new(),
            suggestions: true,
            error_analysis: true,
            privacy_mode: false,
        }
    }
}

impl Default for Web3Config {
    fn default() -> Self {
        Self {
            enabled: false,
            rpc_url: String::new(),
            gas_tracker: true,
            ens_resolver: true,
            auto_detect_projects: true,
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+T".into(),
            close_tab: "Ctrl+W".into(),
            split_horizontal: "Ctrl+Shift+H".into(),
            split_vertical: "Ctrl+Shift+V".into(),
            command_palette: "Ctrl+P".into(),
            ai_panel: "Ctrl+Shift+A".into(),
            settings: "Ctrl+Comma".into(),
        }
    }
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
            keepalive_interval: 30,
            keepalive_max_count: 3,
        }
    }
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            auto_update: true,
            plugins: Vec::new(),
        }
    }
}

// ── Loading ───────────────────────────────────────────────────────────────────

impl ThothConfig {
    /// Load config, searching standard locations. Returns default if none found.
    pub fn load() -> ConfigResult<Self> {
        let paths = config_search_paths();

        for path in &paths {
            if path.exists() {
                debug!("Found config at {}", path.display());
                return Self::load_from(path);
            }
        }

        info!("No config file found, using defaults");
        Ok(Self::default())
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::ReadFailed {
            path: path.to_path_buf(),
            source,
        })?;

        let config = match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str::<Self>(&content)?,
            Some("json") => serde_json::from_str::<Self>(&content)?,
            _ => {
                // Try TOML first, then JSON
                toml::from_str::<Self>(&content)
                    .or_else(|_| serde_json::from_str::<Self>(&content))?
            }
        };

        config.validate()?;
        info!("Loaded config from {}", path.display());
        Ok(config)
    }

    /// Save config to the default user config path (TOML format).
    pub fn save(&self) -> ConfigResult<()> {
        let path = default_config_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::TomlSerialize(e.to_string()))?;

        std::fs::write(&path, content)?;
        debug!("Saved config to {}", path.display());
        Ok(())
    }

    /// Validate config values are in acceptable ranges.
    pub fn validate(&self) -> ConfigResult<()> {
        let size = self.appearance.font_size;
        if !(6.0..=72.0).contains(&size) {
            return Err(ConfigError::InvalidFontSize { size });
        }

        let opacity = self.appearance.opacity;
        if !(0.0..=1.0).contains(&opacity) {
            return Err(ConfigError::InvalidOpacity { opacity });
        }

        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn config_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let mode = RunMode::detect();
    let config_dir = mode.config_dir();

    // Per-project config always wins
    paths.push(PathBuf::from("thothterm.toml"));
    paths.push(PathBuf::from("thothterm.json"));

    // Mode-specific config directory
    paths.push(config_dir.join("thothterm.toml"));
    paths.push(config_dir.join("config.toml"));
    paths.push(config_dir.join("thothterm.json"));

    paths
}

fn default_config_path() -> ConfigResult<PathBuf> {
    let mode = RunMode::detect();
    Ok(mode.config_dir().join("thothterm.toml"))
}

fn detect_default_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        // Prefer PowerShell 7, fall back to Windows PowerShell
        if which_shell("pwsh").is_some() {
            return "pwsh".into();
        }
        "powershell".into()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into())
    }
}

#[cfg(not(target_os = "windows"))]
fn which_shell(_name: &str) -> Option<PathBuf> {
    None
}

#[cfg(target_os = "windows")]
fn which_shell(name: &str) -> Option<PathBuf> {
    let output = std::process::Command::new("where").arg(name).output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(PathBuf::from(path))
    } else {
        None
    }
}
