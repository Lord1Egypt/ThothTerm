# 𓆣 ThothTerm — Cross-Platform Guide

> ThothTerm runs natively on Windows, Linux, and macOS.
> One codebase, one binary per platform, same features everywhere.

---

## Platform Support Matrix

| Platform | GPU Backend | PTY | Installer | Status |
|----------|------------|-----|-----------|--------|
| Windows 10/11 | DX12 (preferred), Vulkan, DX11 | ConPTY | MSI + portable EXE | ✅ Target |
| Linux (X11) | Vulkan | PTY (Unix) | AppImage, .deb, .rpm, Flatpak, Snap, AUR | ✅ Target |
| Linux (Wayland) | Vulkan | PTY (Unix) | Same as X11 | ✅ Target |
| macOS 13+ | Metal | PTY (Unix) | DMG + Homebrew | ✅ Target |
| macOS 12 | Metal | PTY | DMG | ⚠️ Best effort |

---

## Windows 🪟

### GPU Backend: DX12
ThothTerm uses DirectX 12 on Windows by default.
- Available on all Windows 10+ machines
- Stable, well-tested with wgpu
- Falls back to DX11 on older hardware

```powershell
# Force DX12 (set this first, before any other step)
$env:WGPU_BACKEND = "dx12"
```

### PTY: ConPTY
- Windows 10 version 1903+ has ConPTY built-in
- Supports: PowerShell 5, PowerShell 7, CMD, Git Bash, WSL2, fish
- ThothTerm wraps ConPTY for full terminal emulation

### Supported Shells on Windows
```toml
# thothterm.toml
[profiles]
[[profiles.list]]
name = "PowerShell 7"
command = ["pwsh.exe"]
default = true

[[profiles.list]]
name = "Windows PowerShell"
command = ["powershell.exe"]

[[profiles.list]]
name = "CMD"
command = ["cmd.exe"]

[[profiles.list]]
name = "Git Bash"
command = ["C:\\Program Files\\Git\\bin\\bash.exe", "--login", "-i"]

[[profiles.list]]
name = "WSL Ubuntu"
command = ["wsl.exe", "-d", "Ubuntu"]
```

### Windows-Specific Features
- **Quake Mode**: Drop-down terminal on hotkey (like Cmder)
- **Jump List**: Recent sessions in taskbar right-click menu
- **Notification Area**: Optional system tray icon
- **Windows Hello**: Biometric auth for credential manager

### Windows Build Notes
```powershell
# Required: Visual Studio Build Tools 2022
# Required: Windows 10 SDK
# Required: MSVC toolchain (not MinGW)
# Required: WGPU_BACKEND=dx12

rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc --release
```

---

## Linux 🐧

### GPU Backend: Vulkan (preferred), OpenGL fallback
```bash
# Force Vulkan
WGPU_BACKEND=vulkan thothterm

# If Vulkan fails, use OpenGL
WGPU_BACKEND=gl thothterm
```

### Display Server: Wayland + X11
ThothTerm runs natively on both:
```bash
# Wayland (default on Ubuntu 22.04+)
WAYLAND_DISPLAY=wayland-0 thothterm

# X11 (XWayland or pure X11)
DISPLAY=:0 thothterm

# Force X11 even on Wayland
WINIT_UNIX_BACKEND=x11 thothterm
```

### Package Installation Targets
```bash
# Debian/Ubuntu (.deb)
sudo dpkg -i thothterm_0.1.0_amd64.deb

# Fedora/RHEL (.rpm)
sudo rpm -i thothterm-0.1.0.x86_64.rpm

# Arch Linux (AUR)
yay -S thothterm

# Flatpak (sandboxed, works on any distro)
flatpak install io.github.lordegypt.ThothTerm

# Snap
sudo snap install thothterm

# AppImage (portable, no install needed)
chmod +x ThothTerm-0.1.0.AppImage
./ThothTerm-0.1.0.AppImage
```

### Linux Build Dependencies
```bash
# Ubuntu 22.04
sudo apt install -y \
  build-essential cmake pkg-config \
  libssl-dev libxcb1-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libfontconfig1-dev \
  libfreetype6-dev libharfbuzz-dev libssh2-1-dev
```

---

## macOS 🍎

### GPU Backend: Metal
- Metal is Apple's GPU API
- Available on all Macs (Intel + Apple Silicon)
- wgpu uses Metal on macOS — fast and stable

### Build Requirements
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For Apple Silicon (M1/M2/M3):
rustup target add aarch64-apple-darwin

# For Intel Mac:
rustup target add x86_64-apple-darwin

# Universal binary (works on both):
cargo build --target universal-apple-darwin
```

### Distribution
```bash
# Create DMG
cargo install cargo-bundle
cargo bundle --release
# Output: target/release/bundle/osx/ThothTerm.app

# Install via Homebrew (after publishing cask)
brew install --cask thothterm
```

### macOS-Specific Features
- Native macOS menu bar
- Touch Bar support (on compatible MacBooks)
- Spotlight integration
- macOS Notifications API
- Keychain integration for credential manager
- Universal binary (Intel + Apple Silicon)

---

## Plugin Compatibility

Plugins written as WASM are 100% cross-platform by default.
The same `.wasm` file runs on Windows, Linux, and macOS — no recompilation.

```
plugin.wasm  →  Windows ✅ / Linux ✅ / macOS ✅
```

---

## Config File Locations

| Platform | Config Path |
|----------|------------|
| Windows | `%APPDATA%\ThothTerm\thothterm.toml` |
| Linux | `~/.config/thothterm/thothterm.toml` |
| macOS | `~/Library/Application Support/ThothTerm/thothterm.toml` |

ThothTerm also checks the current directory for `thothterm.toml` first,
so you can have per-project configs.

---

## Environment Variables

| Variable | Effect |
|----------|--------|
| `THOTHTERM_CONFIG_FILE` | Override config file path |
| `THOTHTERM_LOG` | Log level: error, warn, info, debug, trace |
| `WGPU_BACKEND` | GPU backend: dx12, vulkan, metal, gl |
| `THOTHTERM_SHELL` | Override default shell |
| `THOTHTERM_THEME` | Override theme name |

---

## Cross-Platform CI (GitHub Actions)

The `.github/workflows/build.yml` file builds ThothTerm for all 3 platforms
on every push. See `PHASE1.md` for the full CI config.

Build artifacts are uploaded automatically:
- `thothterm-windows.exe`
- `thothterm-linux` (AppImage)
- `thothterm-macos.dmg`
