# 𓆣 ThothTerm — Technical Architecture

---

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    𓆣 ThothTerm                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  🎨 UI LAYER (Rust + winit + wgpu)                   │   │
│  │  Tab Bar | Pane Grid | Command Palette | Status Bar   │   │
│  └──────────────────────────────────────────────────────┘   │
│                    ↕ events / render                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  ⚙️ CORE ENGINE (Rust)                               │   │
│  │  PTY Manager | SSH Client | MUX | Terminal Emulator  │   │
│  └──────────────────────────────────────────────────────┘   │
│                    ↕ plugin API                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  🧩 PLUGIN SYSTEM (WASM + wasmtime)                  │   │
│  │  AI Plugin | Web3 Plugin | Theme Engine | Custom     │   │
│  └──────────────────────────────────────────────────────┘   │
│                    ↕ config                                 │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  ⚙️ CONFIG LAYER                                     │   │
│  │  thothterm.toml | thothterm.json | GUI Settings      │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Crate Structure (Rust Workspace)

```
ThothTerm/
├── Cargo.toml                    ← workspace root
├── crates/
│   ├── thothterm/                ← main binary
│   │   └── src/main.rs
│   ├── thothterm-core/           ← core engine (PTY, SSH, MUX)
│   │   └── src/
│   │       ├── pty.rs
│   │       ├── ssh.rs
│   │       ├── mux.rs
│   │       └── term.rs
│   ├── thothterm-gui/            ← UI layer (winit + wgpu)
│   │   └── src/
│   │       ├── window.rs
│   │       ├── tab_bar.rs
│   │       ├── pane_grid.rs
│   │       ├── status_bar.rs
│   │       └── settings_panel.rs
│   ├── thothterm-config/         ← config parsing (TOML/JSON)
│   │   └── src/
│   │       ├── config.rs
│   │       └── defaults.rs
│   ├── thothterm-plugins/        ← WASM plugin runtime
│   │   └── src/
│   │       ├── runtime.rs        ← wasmtime integration
│   │       ├── api.rs            ← plugin API definition
│   │       └── manager.rs        ← install/load/unload
│   └── thothterm-plugin-sdk/     ← SDK for plugin authors
│       └── src/
│           └── lib.rs
```

---

## UI Layer

### Window Management
- **winit**: cross-platform window creation and event loop
- **wgpu**: GPU rendering (DX12 on Windows, Metal on macOS, Vulkan on Linux)
- Each terminal pane gets a GPU canvas region (no per-pane overhead)

### Components
```
ThothWindow
├── TabBar
│   ├── Tab (label, close button, dirty indicator)
│   └── NewTabButton (+)
├── PaneGrid
│   ├── TerminalPane (renders terminal cells via wgpu)
│   ├── SplitHandle (drag to resize)
│   └── PaneControls (title bar, buttons)
├── CommandPalette (Ctrl+P, fuzzy search all commands)
├── StatusBar
│   ├── ShellInfo (current shell, exit code)
│   ├── AIWidget (model name, status)
│   ├── Web3Widget (gas price, wallet address)
│   ├── ResourceWidget (CPU%, RAM)
│   └── ClockWidget
└── SettingsPanel (egui overlay, Ctrl+,)
```

---

## Core Engine

### Terminal Emulator
- Parses ANSI/VT escape sequences (VT520 compatible)
- Maintains screen model: cells, colors, attributes
- Handles scrollback buffer (configurable size)
- Supports: 24-bit color, hyperlinks, images (Sixel, iTerm2 protocol), Unicode, BiDi

### PTY Manager
```rust
pub struct PtyManager {
    pub fn spawn_local(shell: ShellConfig) -> Result<PtyPair>
    pub fn spawn_ssh(config: SshConfig) -> Result<PtyPair>
    pub fn spawn_serial(port: SerialConfig) -> Result<PtyPair>
    pub fn spawn_docker(container: &str, cmd: &str) -> Result<PtyPair>
}
```

### SSH Client
- Uses `ssh2` crate (libssh2 bindings)
- Supports: password, key-based, agent, jump hosts
- SFTP file browser built-in
- Connection profiles stored in `thothterm.toml`

### Multiplexer (MUX)
- Persistent sessions (survive terminal close)
- Attach/detach sessions
- Remote MUX (share sessions over network)
- tmux-style protocol for compatibility

---

## Plugin System

### How Plugins Work
```
Plugin (any language) → compile to WASM → thothterm loads it → runs in sandbox
```

### Plugin Lifecycle
```
1. User runs: thothterm plugin install github.com/user/myplugin
2. ThothTerm downloads, verifies, builds WASM
3. Plugin loaded into wasmtime sandbox
4. Plugin calls ThothTerm API via host functions
5. ThothTerm calls plugin hooks on events
```

### Plugin API (Host Functions available to WASM)
```
// Write to terminal
thoth_write(text: *const u8, len: usize)

// Get current command
thoth_get_current_command() -> *const u8

// Add status bar item
thoth_status_bar_set(slot: u32, text: *const u8, len: usize)

// Show notification
thoth_notify(title: *const u8, body: *const u8)

// HTTP request (if permission granted)
thoth_http_get(url: *const u8) -> *const u8
```

### Plugin Hooks (called by ThothTerm)
```
// Called when terminal output changes
on_output(line: *const u8, len: usize)

// Called on each keystroke
on_key(key_code: u32, modifiers: u32)

// Called when new session starts
on_session_start(session_id: u32)

// Called every second (for status bar updates)
on_tick()
```

---

## AI Integration

```
ThothTerm
    ↓ HTTP
Ollama (localhost:11434)   ← local, private
    OR
OpenAI / Claude API        ← cloud, needs API key
```

### AI Context sent with each request
- Current shell (bash/zsh/fish/pwsh)
- Last 10 commands
- Current directory
- Last error message (if any)
- OS type

---

## Web3 Integration

```
ThothTerm
    ↓ alloy-rs
Ethereum RPC (HTTP/WS)
    ↓
Mainnet / L2 / Local (Anvil/Ganache)
```

### Wallet Security
- Private keys: AES-256-GCM encrypted, stored in OS keychain
- Never stored in plaintext
- Require passphrase on each session

---

## Config Format (`thothterm.toml`)

```toml
[general]
default_shell = "powershell"    # or "bash", "zsh", "fish"
startup_wizard = false          # set to true on first run

[appearance]
font_family = "JetBrains Mono"
font_size = 14.0
theme = "catppuccin-mocha"
opacity = 0.95
background_image = ""
cursor_style = "block"          # block, underline, bar

[keybindings]
new_tab = "Ctrl+T"
close_tab = "Ctrl+W"
split_horizontal = "Ctrl+Shift+H"
split_vertical = "Ctrl+Shift+V"
command_palette = "Ctrl+P"
ai_panel = "Ctrl+Shift+A"

[ai]
enabled = true
provider = "ollama"             # ollama, openai, claude
model = "llama3.2"
base_url = "http://localhost:11434"
api_key = ""
suggestions = true
error_analysis = true
privacy_mode = false

[web3]
enabled = false
rpc_url = "https://eth-mainnet.alchemyapi.io/v2/YOUR_KEY"
gas_tracker = true
ens_resolver = true

[ssh.profiles]
# [[ssh.profiles]]
# name = "my-server"
# host = "192.168.1.1"
# user = "root"
# key = "~/.ssh/id_rsa"

[plugins]
# [[plugins]]
# name = "ai-assistant"
# enabled = true
```

---

## Performance Targets

| Metric | Target | WezTerm baseline |
|--------|--------|-----------------|
| Cold startup | < 300ms | ~400ms |
| RAM idle (1 tab) | < 30MB | ~40MB |
| RAM (10 tabs) | < 80MB | ~100MB |
| Render latency | < 8ms | ~10ms |
| Scrollback (1M lines) | smooth 60fps | smooth |
| SSH connect | < 1s | ~1s |
