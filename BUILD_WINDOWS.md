# 𓆣 ThothTerm — Windows Build Guide

> Complete step-by-step guide to build ThothTerm on Windows 10/11.
> Copy-paste every command into PowerShell (run as Administrator where noted).

---

## Prerequisites Checklist

Run this in PowerShell to check what you have:
```powershell
rustc --version
cargo --version
git --version
cl.exe      # Visual Studio C++ compiler
```

---

## Step 1: Install Rust

```powershell
# Download and run rustup
winget install Rustlang.Rustup
# OR download from https://rustup.rs

# After install, restart PowerShell, then:
rustup toolchain install stable
rustup default stable
rustup target add x86_64-pc-windows-msvc
rustup component add rustfmt clippy

# Verify
rustc --version   # should be 1.78+
cargo --version
```

---

## Step 2: Install Visual Studio Build Tools

ThothTerm requires MSVC linker (not MinGW).

```powershell
# Install Visual Studio Build Tools 2022 (free)
winget install Microsoft.VisualStudio.2022.BuildTools

# During install, select:
# ✅ Desktop development with C++
# ✅ Windows 10/11 SDK (latest)
# ✅ MSVC v143 build tools
```

OR install full Visual Studio 2022 Community (free):
```powershell
winget install Microsoft.VisualStudio.2022.Community
```

---

## Step 3: Install Git

```powershell
winget install Git.Git

# Configure git
git config --global user.name "Lord1Egypt"
git config --global user.email "akim.221992@gmail.com"
```

---

## Step 4: Install sccache (Speeds up Rust builds significantly)

```powershell
cargo install sccache

# Add to your PowerShell profile:
$env:RUSTC_WRAPPER = "sccache"

# Or add permanently to system environment variables:
[System.Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "User")
```

---

## Step 5: Fork and Clone ThothTerm

```powershell
# 1. Go to https://github.com/wez/wezterm
# 2. Click Fork → your account (lordegypt/ThothTerm)
# 3. Clone YOUR fork (not the original)

git clone https://github.com/lordegypt/ThothTerm
cd ThothTerm

# Add upstream for future updates from original
git remote add upstream https://github.com/wez/wezterm
git fetch upstream
```

---

## Step 6: Set Windows GPU Backend (IMPORTANT)

```powershell
# MUST DO THIS — prevents DirectX/Vulkan errors on Windows
# Add to your PowerShell session (or add to system env vars):
$env:WGPU_BACKEND = "dx12"

# Permanent setting:
[System.Environment]::SetEnvironmentVariable("WGPU_BACKEND", "dx12", "User")
```

> ⚠️ **Why**: wgpu on Windows defaults to Vulkan which fails on many Intel/AMD GPUs.
> DX12 is stable and available on all Windows 10+ machines.

---

## Step 7: First Build (Debug)

```powershell
cd ThothTerm

# This first build takes 10-20 minutes (downloads + compiles all deps)
# Get coffee ☕
cargo build 2>&1 | Tee-Object build_log.txt

# If build succeeds, run it:
cargo run --bin thothterm
```

---

## Step 8: Release Build

```powershell
# Optimized build (takes longer but produces fast binary)
cargo build --release

# Binary will be at:
# ThothTerm\target\release\thothterm.exe
```

---

## Step 9: Create Windows Installer (MSI)

```powershell
# Install cargo-wix
cargo install cargo-wix

# Initialize WiX config (first time only)
cargo wix init --force

# Build MSI
cargo wix build

# MSI will be at:
# ThothTerm\target\wix\thothterm-*.msi
```

---

## Common Errors and Fixes

### Error: `LINK : fatal error LNK1181: cannot open input file 'openssl.lib'`
```powershell
# Install OpenSSL via vcpkg
winget install Microsoft.Vcpkg
vcpkg install openssl:x64-windows-static

# Set env vars:
$env:OPENSSL_DIR = "C:\vcpkg\installed\x64-windows-static"
$env:OPENSSL_STATIC = "1"
```

### Error: `error: linker 'link.exe' not found`
```powershell
# Visual Studio Build Tools not in PATH
# Open "Developer PowerShell for VS 2022" and retry
# Or run:
& "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
```

### Error: `wgpu: no suitable adapter found`
```powershell
# Force DX12 backend
$env:WGPU_BACKEND = "dx12"
cargo run --bin thothterm
```

### Error: `thread 'main' panicked at 'called Result::unwrap() on Err ...'` (first run)
```powershell
# Likely missing config directory, create it:
mkdir "$env:APPDATA\ThothTerm"
# Then retry
```

### Error: `cargo build` hangs forever
```powershell
# Check if antivirus is scanning Rust build artifacts
# Add these to Windows Defender exclusions:
# - C:\Users\YourName\.cargo
# - ThothTerm\target\
```

---

## Development Workflow

```powershell
# Run with logging (shows debug output)
$env:THOTHTERM_LOG = "debug"
cargo run --bin thothterm

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Build specific crate only
cargo build -p thothterm-core
```

---

## Useful PowerShell Aliases (add to your profile)

```powershell
# Add to: C:\Users\YourName\Documents\PowerShell\Microsoft.PowerShell_profile.ps1

Set-Alias -Name tt -Value "cargo run --bin thothterm --"
function tt-build { cargo build --release }
function tt-test  { cargo test }
function tt-check { cargo check && cargo clippy }
function tt-log   { $env:THOTHTERM_LOG = "debug"; cargo run --bin thothterm }
```

---

## IDE Setup (VS Code)

```powershell
# Install VS Code
winget install Microsoft.VisualStudioCode

# Install Rust extensions:
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb     # debugger
code --install-extension tamasfe.even-better-toml

# Open project
code ThothTerm\
```

**Recommended VS Code settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

---

## Notes for Mohamed Specifically

- You already have Rust from the **ethsmith** project — just update it: `rustup update`
- Use the same GitHub account (`lordegypt`) you used for ethsmith
- The `WGPU_BACKEND=dx12` env var is the #1 thing to set on Windows — do it first
- First build will take a long time because of all the upstream dependencies — this is normal
- Use `cargo check` instead of `cargo build` while iterating — it's much faster
