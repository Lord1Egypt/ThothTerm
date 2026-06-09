use std::path::PathBuf;

/// Whether ThothTerm is running in installed or portable mode.
///
/// **Portable mode** is activated when a `.portable` file exists in the same
/// directory as the running executable. In portable mode, ALL data (config,
/// plugins, logs) is stored next to the executable instead of in OS-standard
/// directories. This makes the whole installation self-contained and suitable
/// for USB drives, enterprise deployments without admin rights, or side-by-side
/// testing.
///
/// **Installed mode** (default) stores data in the OS-standard directories:
/// - Linux/macOS: `~/.config/thothterm/`, `~/.local/share/thothterm/`
/// - Windows: `%APPDATA%\ThothTerm\`
#[derive(Debug, Clone)]
pub enum RunMode {
    Installed,
    Portable { base: PathBuf },
}

impl RunMode {
    /// Detect the run mode by checking for a `.portable` marker next to the exe.
    pub fn detect() -> Self {
        // Allow env-var override for testing
        if std::env::var_os("THOTHTERM_PORTABLE").is_some() {
            if let Ok(base) = std::env::current_dir() {
                return Self::Portable { base };
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                if dir.join(".portable").exists() {
                    return Self::Portable {
                        base: dir.to_path_buf(),
                    };
                }
            }
        }

        Self::Installed
    }

    /// True when running from a portable installation.
    pub fn is_portable(&self) -> bool {
        matches!(self, Self::Portable { .. })
    }

    /// Directory for user configuration files.
    pub fn config_dir(&self) -> PathBuf {
        match self {
            Self::Installed => dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("thothterm"),
            Self::Portable { base } => base.join("config"),
        }
    }

    /// Directory where plugins are installed.
    pub fn plugins_dir(&self) -> PathBuf {
        match self {
            Self::Installed => self.config_dir().join("plugins"),
            Self::Portable { base } => base.join("plugins"),
        }
    }

    /// Directory for application data (logs, cache, wallet store).
    pub fn data_dir(&self) -> PathBuf {
        match self {
            Self::Installed => dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("thothterm"),
            Self::Portable { base } => base.join("data"),
        }
    }

    /// Directory for log files.
    pub fn log_dir(&self) -> PathBuf {
        self.data_dir().join("logs")
    }

    /// Wallet store path.
    pub fn wallet_store_path(&self) -> PathBuf {
        self.data_dir().join("wallets.json")
    }

    /// Print a one-line description for diagnostics.
    pub fn description(&self) -> String {
        match self {
            Self::Installed => "installed (OS directories)".into(),
            Self::Portable { base } => {
                format!("portable ({})", base.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_mode_gives_system_paths() {
        // In test env there's no .portable file next to the test binary (usually),
        // but we just check the shape of the paths.
        let mode = RunMode::Installed;
        let config = mode.config_dir();
        assert!(config.ends_with("thothterm"));
        let plugins = mode.plugins_dir();
        assert!(plugins.ends_with("plugins"));
    }

    #[test]
    fn portable_mode_uses_base_dir() {
        let base = PathBuf::from("/tmp/thothterm-portable");
        let mode = RunMode::Portable { base: base.clone() };
        assert_eq!(mode.config_dir(), base.join("config"));
        assert_eq!(mode.plugins_dir(), base.join("plugins"));
        assert_eq!(mode.data_dir(), base.join("data"));
        assert_eq!(mode.wallet_store_path(), base.join("data").join("wallets.json"));
        assert!(mode.is_portable());
    }

    #[test]
    fn installed_mode_is_not_portable() {
        assert!(!RunMode::Installed.is_portable());
    }
}
