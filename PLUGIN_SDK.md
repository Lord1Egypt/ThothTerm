# 𓆣 ThothTerm — Plugin Developer Guide

> How to write ThothTerm plugins in any language that compiles to WASM.

---

## Overview

ThothTerm plugins are WASM modules. You can write them in:
- **Rust** (recommended, best performance)
- **Go** (via TinyGo)
- **AssemblyScript** (TypeScript-like)
- **C/C++** (via Emscripten)

---

## Quick Start (Rust Plugin)

```bash
# Install WASM target
rustup target add wasm32-wasi

# Create new plugin
cargo new my-thothterm-plugin --lib
cd my-thothterm-plugin
```

`Cargo.toml`:
```toml
[package]
name = "my-thothterm-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
thothterm-plugin-sdk = "0.1"
```

`src/lib.rs`:
```rust
use thothterm_plugin_sdk::*;

// Called when plugin loads
#[no_mangle]
pub extern "C" fn plugin_init() {
    thoth_notify("My Plugin", "Hello from ThothTerm plugin!");
}

// Called on each line of terminal output
#[no_mangle]
pub extern "C" fn on_output(ptr: *const u8, len: usize) {
    let line = unsafe { std::str::from_raw_parts(ptr, len) };
    if line.contains("error") {
        thoth_status_bar_set(0, "⚠️ Error detected");
    }
}

// Called every second (for status bar updates)
#[no_mangle]
pub extern "C" fn on_tick() {
    // Update status bar with timestamp
    let time = thoth_get_time();
    thoth_status_bar_set(1, &format!("🕐 {}", time));
}
```

Build:
```bash
cargo build --release --target wasm32-wasi
# Output: target/wasm32-wasi/release/my_thothterm_plugin.wasm
```

---

## Plugin Manifest (`plugin.toml`)

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
author = "Your Name"
description = "What this plugin does"
homepage = "https://github.com/you/my-plugin"
api_version = "1.0"
entry = "my_thothterm_plugin.wasm"

# What hooks this plugin uses
hooks = [
    "on_output",
    "on_tick",
    "on_key",
    "on_session_start",
]

# Permissions required (user will be prompted to approve)
permissions = [
    "status_bar",       # write to status bar
    "notification",     # show desktop notifications
    "network",          # make HTTP requests
    "clipboard",        # read/write clipboard
    "filesystem:read",  # read files
]
```

---

## Full Plugin API Reference

### Host Functions (call from plugin → ThothTerm)

```rust
// Write text to the current terminal
fn thoth_write(text: &str);

// Read the last N lines of terminal output
fn thoth_get_output(lines: u32) -> Vec<String>;

// Get current shell command being typed
fn thoth_get_current_input() -> String;

// Update a status bar slot (slot 0-9)
fn thoth_status_bar_set(slot: u32, text: &str);

// Show a desktop notification
fn thoth_notify(title: &str, body: &str);

// Make an HTTP GET request (requires "network" permission)
fn thoth_http_get(url: &str) -> String;

// Make an HTTP POST request
fn thoth_http_post(url: &str, body: &str, content_type: &str) -> String;

// Open a floating panel/dialog
fn thoth_panel_open(title: &str, content: &str);
fn thoth_panel_close();

// Read/write clipboard (requires "clipboard" permission)
fn thoth_clipboard_get() -> String;
fn thoth_clipboard_set(text: &str);

// Get current working directory
fn thoth_get_cwd() -> String;

// Get ThothTerm version
fn thoth_get_version() -> String;

// Get current time (Unix timestamp)
fn thoth_get_time() -> u64;

// Log a message (shown in ThothTerm debug logs)
fn thoth_log(level: &str, msg: &str);

// Store plugin-specific config value
fn thoth_config_set(key: &str, value: &str);
fn thoth_config_get(key: &str) -> Option<String>;
```

### Plugin Hooks (ThothTerm calls → your plugin)

```rust
// Called when plugin first loads
#[no_mangle]
pub extern "C" fn plugin_init() { }

// Called when plugin is about to unload
#[no_mangle]
pub extern "C" fn plugin_cleanup() { }

// Called on each line of terminal output
#[no_mangle]
pub extern "C" fn on_output(ptr: *const u8, len: usize) { }

// Called on each keystroke
// key_code: VirtualKey code, modifiers: bitmask (Ctrl=1, Alt=2, Shift=4)
#[no_mangle]
pub extern "C" fn on_key(key_code: u32, modifiers: u32) { }

// Called when new PTY session starts (new tab, new window)
#[no_mangle]
pub extern "C" fn on_session_start(session_id: u32) { }

// Called when session ends (tab closed)
#[no_mangle]
pub extern "C" fn on_session_end(session_id: u32) { }

// Called every second
#[no_mangle]
pub extern "C" fn on_tick() { }

// Called when SSH connection established
#[no_mangle]
pub extern "C" fn on_ssh_connect(host_ptr: *const u8, host_len: usize) { }

// Called when SSH disconnects
#[no_mangle]
pub extern "C" fn on_ssh_disconnect(host_ptr: *const u8, host_len: usize) { }

// Called when command finishes (exit code available)
#[no_mangle]
pub extern "C" fn on_command_exit(exit_code: i32) { }
```

---

## Example Plugins

### 1. Git Status Bar Plugin

Shows current git branch and status in the status bar.

```rust
use thothterm_plugin_sdk::*;

#[no_mangle]
pub extern "C" fn on_tick() {
    let cwd = thoth_get_cwd();
    if let Ok(branch) = get_git_branch(&cwd) {
        let status = get_git_status(&cwd);
        let icon = if status.is_clean() { "✅" } else { "📝" };
        thoth_status_bar_set(2, &format!("{} {}", icon, branch));
    }
}
```

### 2. Error AI Analyzer Plugin

When a command fails, automatically ask AI to explain the error.

```rust
#[no_mangle]
pub extern "C" fn on_command_exit(exit_code: i32) {
    if exit_code != 0 {
        let output = thoth_get_output(20);
        let error_lines = output.join("\n");

        // Call local Ollama
        let response = thoth_http_post(
            "http://localhost:11434/api/generate",
            &format!(r#"{{"model":"llama3.2","prompt":"Explain this error and how to fix it:\n{error_lines}","stream":false}}"#),
            "application/json"
        );

        thoth_panel_open("🤖 AI Error Analysis", &response);
    }
}
```

### 3. Web3 Gas Tracker Plugin

Shows ETH gas price in status bar.

```rust
#[no_mangle]
pub extern "C" fn on_tick() {
    // Every 30 seconds
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        if COUNTER % 30 != 0 { return; }
    }

    let response = thoth_http_get(
        "https://api.etherscan.io/api?module=gastracker&action=gasoracle"
    );
    // Parse JSON, extract gas price
    thoth_status_bar_set(5, &format!("⛽ {}gwei", gas_price));
}
```

---

## Installing Plugins

```bash
# From GitHub
thothterm plugin install github.com/user/plugin-name

# From local path (development)
thothterm plugin install ./path/to/plugin

# From URL
thothterm plugin install https://example.com/plugin.wasm

# List installed plugins
thothterm plugin list

# Enable/disable
thothterm plugin enable my-plugin
thothterm plugin disable my-plugin

# Update all plugins
thothterm plugin update

# Remove plugin
thothterm plugin remove my-plugin
```

---

## Publishing Your Plugin

1. Create GitHub repo: `thothterm-plugin-yourname`
2. Add `plugin.toml` and the `.wasm` file to releases
3. Users install with: `thothterm plugin install github.com/you/thothterm-plugin-yourname`
4. Submit to ThothTerm plugin registry (coming in v1.0): open a PR to the registry repo

---

## SDK Helper (thothterm-plugin-sdk)

Add to `Cargo.toml`:
```toml
[dependencies]
thothterm-plugin-sdk = "0.1"
```

The SDK provides safe Rust wrappers around all the raw FFI functions above.
Source: `crates/thothterm-plugin-sdk/src/lib.rs`
