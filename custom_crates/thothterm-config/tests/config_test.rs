use thothterm_config::*;
use tempfile::tempdir;
use std::fs;

#[test]
fn default_config_is_valid() {
    let config = ThothConfig::default();
    assert!(config.validate().is_ok(), "Default config must always be valid");
}

#[test]
fn load_from_toml_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("thothterm.toml");

    fs::write(&path, r#"
[general]
scrollback_lines = 50000

[appearance]
font_family = "Fira Code"
font_size = 16.0
theme = "gruvbox-dark"
opacity = 0.9

[ai]
enabled = true
provider = "ollama"
model = "llama3.2"
    "#).unwrap();

    let config = ThothConfig::load_from(&path).unwrap();
    assert_eq!(config.general.scrollback_lines, 50000);
    assert_eq!(config.appearance.font_family, "Fira Code");
    assert_eq!(config.appearance.font_size, 16.0);
    assert_eq!(config.appearance.theme, "gruvbox-dark");
    assert!((config.appearance.opacity - 0.9).abs() < f32::EPSILON);
    assert!(config.ai.enabled);
    assert_eq!(config.ai.model, "llama3.2");
}

#[test]
fn load_from_json_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("thothterm.json");

    fs::write(&path, r#"{
        "general": { "scrollback_lines": 20000 },
        "appearance": { "font_size": 12.0 }
    }"#).unwrap();

    let config = ThothConfig::load_from(&path).unwrap();
    assert_eq!(config.general.scrollback_lines, 20000);
    assert_eq!(config.appearance.font_size, 12.0);
}

#[test]
fn invalid_font_size_rejected() {
    let mut config = ThothConfig::default();
    config.appearance.font_size = 3.0; // below minimum of 6.0
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("font size"));
}

#[test]
fn invalid_opacity_rejected() {
    let mut config = ThothConfig::default();
    config.appearance.opacity = 1.5; // above maximum of 1.0
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("opacity"));
}

#[test]
fn save_and_reload_config() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("thothterm.toml");

    let original = ThothConfig {
        general: GeneralConfig {
            scrollback_lines: 99_999,
            startup_wizard: false,
            ..GeneralConfig::default()
        },
        appearance: AppearanceConfig {
            font_size: 18.0,
            theme: "nord".into(),
            ..AppearanceConfig::default()
        },
        ..ThothConfig::default()
    };

    // Serialize to TOML and write manually (save() uses OS config dir)
    let toml_str = toml::to_string_pretty(&original).unwrap();
    fs::write(&path, toml_str).unwrap();

    let reloaded = ThothConfig::load_from(&path).unwrap();
    assert_eq!(reloaded.general.scrollback_lines, 99_999);
    assert!(!reloaded.general.startup_wizard);
    assert_eq!(reloaded.appearance.font_size, 18.0);
    assert_eq!(reloaded.appearance.theme, "nord");
}

#[test]
fn malformed_toml_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.toml");
    fs::write(&path, "this is not valid toml ][[[").unwrap();

    let result = ThothConfig::load_from(&path);
    assert!(result.is_err());
}

#[test]
fn malformed_json_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.json");
    fs::write(&path, "{ not valid json }}}").unwrap();

    let result = ThothConfig::load_from(&path);
    assert!(result.is_err());
}

#[test]
fn missing_file_returns_not_found_error() {
    let result = ThothConfig::load_from(std::path::Path::new("/does/not/exist.toml"));
    assert!(matches!(result, Err(thothterm_config::error::ConfigError::ReadFailed { .. })));
}

#[test]
fn partial_toml_uses_defaults_for_missing_fields() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("partial.toml");

    // Only specify font_size — everything else should use defaults
    fs::write(&path, "[appearance]\nfont_size = 20.0\n").unwrap();

    let config = ThothConfig::load_from(&path).unwrap();
    assert_eq!(config.appearance.font_size, 20.0);
    // Default values still present
    assert_eq!(config.appearance.cursor_style, CursorStyle::Block);
    assert!(!config.ai.enabled);
    assert!(!config.web3.enabled);
}

#[test]
fn ssh_profiles_parsed_correctly() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ssh.toml");

    fs::write(&path, r#"
[[ssh.profiles]]
name = "prod-server"
host = "192.168.1.100"
port = 22
user = "admin"
key_path = "/home/user/.ssh/prod_key"

[[ssh.profiles]]
name = "jump-host"
host = "jump.example.com"
port = 2222
user = "deploy"
    "#).unwrap();

    let config = ThothConfig::load_from(&path).unwrap();
    assert_eq!(config.ssh.profiles.len(), 2);
    assert_eq!(config.ssh.profiles[0].name, "prod-server");
    assert_eq!(config.ssh.profiles[0].port, 22);
    assert_eq!(config.ssh.profiles[1].port, 2222);
}
