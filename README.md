# 𓆣 ThothTerm

<p align="center">
  <strong>A GPU-accelerated terminal emulator for the next 10 years</strong><br>
  AI-native · Web3-native · WASM-plugin-extensible · Cross-platform
</p>

<p align="center">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/GPU-wgpu-blueviolet?style=for-the-badge" alt="wgpu">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=for-the-badge" alt="cross-platform">
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="MIT">
</p>

<p align="center">
  <img src="demo.gif" alt="ThothTerm demo" width="800">
</p>

---

Named after **Thoth** — Egyptian god of writing, wisdom, and knowledge — because a terminal is where you write the future.

## Features

| Feature | Status |
|---------|--------|
| GPU rendering (wgpu — DX12/Metal/Vulkan) | ✅ |
| True color, ligatures, font fallback | ✅ |
| SSH multiplexer + tabs + panes | ✅ |
| TOML/JSON config (no Lua required) | ✅ |
| GUI Settings panel (Ctrl+,) | ✅ |
| WASM plugin system (any language) | 🚧 Phase 2 |
| AI assistant (Ollama + OpenAI/Claude) | 🚧 Phase 3 |
| Web3 / Ethereum integration | 🚧 Phase 4 |
| Windows MSI + Linux AppImage + macOS DMG | 🚧 Phase 5 |

## Installation

Binaries will be available on the [Releases](https://github.com/Lord1Egypt/ThothTerm/releases) page once v1.0 is published.

**Build from source (Linux):**

```bash
sudo apt install -y build-essential pkg-config libssl-dev libxcb1-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libfontconfig1-dev libfreetype6-dev

git clone https://github.com/Lord1Egypt/ThothTerm.git
cd ThothTerm
cargo build --release
./target/release/thothterm
```

See [BUILD_LINUX.md](BUILD_LINUX.md) for the full guide.  
See [BUILD_WINDOWS.md](BUILD_WINDOWS.md) for Windows (DX12 backend).

## Configuration

ThothTerm reads `thothterm.toml` from:
1. Current directory
2. `~/.config/thothterm/thothterm.toml`

Example config:

```toml
[general]
default_shell = "/bin/bash"

[appearance]
font_size = 14.0
window_background_opacity = 0.95
color_scheme = "Tokyo Night"

[ai]
enabled = true
provider = "ollama"
ollama_url = "http://localhost:11434"
```

Open the settings panel any time with **Ctrl+,**.

## WASM Plugins

ThothTerm supports plugins written in any language that compiles to WASM.

```bash
thothterm plugin install ./my-plugin/
thothterm plugin list
thothterm plugin enable my-plugin
```

See [PLUGIN_SDK.md](PLUGIN_SDK.md) for the full SDK guide.

## AI Assistant

With Ollama running locally, ThothTerm can:
- Explain error messages
- Convert natural language to shell commands
- Suggest commands as you type

No API key needed for local AI. Optional OpenAI/Claude API for cloud mode.

## Roadmap

See [PLAN.md](PLAN.md) for the full phased roadmap and [PROGRESS.md](PROGRESS.md) for current status.

## Contributing

Issues and PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT — see [LICENSE.md](LICENSE.md).

---

> Built by [Mohamed Mounir (Lord1Egypt)](https://github.com/Lord1Egypt)  
> Base: GPU rendering engine derived from [wez/wezterm](https://github.com/wez/wezterm) — thank you to Wez Furlong and all WezTerm contributors.
