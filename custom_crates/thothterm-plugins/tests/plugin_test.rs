use thothterm_plugins::{PluginManager, PluginManifest};
use tempfile::tempdir;
use std::fs;

fn make_valid_plugin_dir(base: &std::path::Path, name: &str) -> std::path::PathBuf {
    let dir = base.join(name);
    fs::create_dir_all(&dir).unwrap();

    // Create a fake WASM file
    fs::write(dir.join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();

    // Create plugin.toml
    fs::write(dir.join("plugin.toml"), format!(r#"
[plugin]
name = "{name}"
version = "0.1.0"
author = "Test Author"
description = "A test plugin"
entry = "plugin.wasm"
api_version = "1.0"

hooks = ["on_output", "on_tick"]
permissions = ["status_bar"]
    "#)).unwrap();

    dir
}

#[test]
fn load_valid_manifest() {
    let tmp = tempdir().unwrap();
    let plugin_dir = make_valid_plugin_dir(tmp.path(), "my-plugin");

    let manifest = PluginManifest::load(&plugin_dir).unwrap();
    assert_eq!(manifest.plugin.name, "my-plugin");
    assert_eq!(manifest.plugin.version, "0.1.0");
    assert_eq!(manifest.plugin.api_version, "1.0");
    assert!(manifest.has_hook("on_output"));
    assert!(manifest.has_hook("on_tick"));
    assert!(!manifest.has_hook("on_key")); // not declared
    assert!(manifest.has_permission("status_bar"));
    assert!(!manifest.has_permission("network")); // not declared
}

#[test]
fn missing_plugin_toml_returns_error() {
    let tmp = tempdir().unwrap();
    let result = PluginManifest::load(tmp.path());
    assert!(matches!(result, Err(thothterm_plugins::error::PluginError::InvalidManifest { .. })));
}

#[test]
fn missing_wasm_file_returns_error() {
    let tmp = tempdir().unwrap();
    // plugin.toml exists but references a WASM that doesn't exist
    fs::write(tmp.path().join("plugin.toml"), r#"
[plugin]
name = "bad-plugin"
version = "0.1.0"
author = "Test"
description = "Bad plugin"
entry = "nonexistent.wasm"
api_version = "1.0"
    "#).unwrap();

    let result = PluginManifest::load(tmp.path());
    assert!(matches!(result, Err(thothterm_plugins::error::PluginError::WasmNotFound { .. })));
}

#[test]
fn incompatible_api_version_rejected() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("plugin.wasm"), b"\0asm").unwrap();
    fs::write(tmp.path().join("plugin.toml"), r#"
[plugin]
name = "future-plugin"
version = "2.0.0"
author = "Future Dev"
description = "Plugin for API v2"
entry = "plugin.wasm"
api_version = "2.0"
    "#).unwrap();

    let result = PluginManifest::load(tmp.path());
    assert!(matches!(result, Err(thothterm_plugins::error::PluginError::ApiVersionMismatch { .. })));
}

#[test]
fn install_and_list_plugins() {
    let plugins_dir = tempdir().unwrap();
    let plugin_src = tempdir().unwrap();
    make_valid_plugin_dir(plugin_src.path(), "test-plugin");
    let src_dir = plugin_src.path().join("test-plugin");

    let mut manager = PluginManager::new(plugins_dir.path());
    let name = manager.install_local(&src_dir).unwrap();
    assert_eq!(name, "test-plugin");

    let list = manager.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "test-plugin");
    assert!(list[0].enabled);
}

#[test]
fn install_same_plugin_twice_errors() {
    let plugins_dir = tempdir().unwrap();
    let plugin_src = tempdir().unwrap();
    make_valid_plugin_dir(plugin_src.path(), "dupe-plugin");
    let src_dir = plugin_src.path().join("dupe-plugin");

    let mut manager = PluginManager::new(plugins_dir.path());
    manager.install_local(&src_dir).unwrap();

    let result = manager.install_local(&src_dir);
    assert!(matches!(result, Err(thothterm_plugins::error::PluginError::AlreadyInstalled { .. })));
}

#[test]
fn enable_disable_plugin() {
    let plugins_dir = tempdir().unwrap();
    let plugin_src = tempdir().unwrap();
    make_valid_plugin_dir(plugin_src.path(), "toggle-plugin");
    let src_dir = plugin_src.path().join("toggle-plugin");

    let mut manager = PluginManager::new(plugins_dir.path());
    manager.install_local(&src_dir).unwrap();

    assert!(manager.is_enabled("toggle-plugin"));
    manager.disable("toggle-plugin").unwrap();
    assert!(!manager.is_enabled("toggle-plugin"));
    manager.enable("toggle-plugin").unwrap();
    assert!(manager.is_enabled("toggle-plugin"));
}

#[test]
fn remove_plugin() {
    let plugins_dir = tempdir().unwrap();
    let plugin_src = tempdir().unwrap();
    make_valid_plugin_dir(plugin_src.path(), "remove-plugin");
    let src_dir = plugin_src.path().join("remove-plugin");

    let mut manager = PluginManager::new(plugins_dir.path());
    manager.install_local(&src_dir).unwrap();
    assert_eq!(manager.plugin_count(), 1);

    manager.remove("remove-plugin").unwrap();
    assert_eq!(manager.plugin_count(), 0);

    // Should also be gone from disk
    assert!(!plugins_dir.path().join("remove-plugin").exists());
}

#[test]
fn remove_nonexistent_plugin_errors() {
    let plugins_dir = tempdir().unwrap();
    let mut manager = PluginManager::new(plugins_dir.path());
    let result = manager.remove("ghost-plugin");
    assert!(matches!(result, Err(thothterm_plugins::error::PluginError::NotFound { .. })));
}

#[test]
fn empty_plugin_dir_loads_zero() {
    let plugins_dir = tempdir().unwrap();
    let mut manager = PluginManager::new(plugins_dir.path());
    let results = manager.load_all();
    assert!(results.is_empty());
}
