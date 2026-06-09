# 𓆣 ThothTerm — Linux Build Guide

> Tested on Ubuntu 22.04 / Debian 12. Should work on most distros.

---

## Step 1: Install System Dependencies

```bash
# Ubuntu / Debian
sudo apt update && sudo apt install -y \
  build-essential cmake pkg-config git curl \
  libssl-dev \
  libx11-dev libx11-xcb-dev \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxcb-icccm4-dev libxcb-keysyms1-dev libxcb-util-dev \
  libxcb-image0-dev libxcb-randr0-dev libxcb-ewmh-dev \
  libxkbcommon-dev libxkbcommon-x11-dev \
  libfontconfig1-dev libfreetype6-dev \
  libharfbuzz-dev libssh2-1-dev libzstd-dev \
  libwayland-dev \
  python3

# Fedora / RHEL
sudo dnf install -y \
  gcc gcc-c++ cmake openssl-devel \
  libxcb-devel libxkbcommon-devel \
  fontconfig-devel freetype-devel \
  harfbuzz-devel libssh2-devel \
  zstd-devel python3 git curl

# Arch Linux
sudo pacman -S --noconfirm \
  base-devel cmake openssl \
  libxcb libxkbcommon \
  fontconfig freetype2 \
  harfbuzz libssh2 zstd \
  python git
```

---

## Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy

rustc --version   # should be 1.78+
```

---

## Step 3: Clone ThothTerm

```bash
git clone https://github.com/lordegypt/ThothTerm
cd ThothTerm
git remote add upstream https://github.com/wez/wezterm
```

---

## Step 4: Build

```bash
# Debug build (faster to compile, slower to run)
cargo build

# Release build
cargo build --release

# Run
cargo run --bin thothterm
```

---

## Step 5: Create AppImage

```bash
# Install cargo-appimage
cargo install cargo-appimage

# Build AppImage
cargo appimage

# Output: target/appimage/thothterm.AppImage
chmod +x target/appimage/thothterm.AppImage
./target/appimage/thothterm.AppImage
```

---

## Step 6: Install System-wide

```bash
# Copy binary
sudo cp target/release/thothterm /usr/local/bin/

# Create desktop entry
cat > ~/.local/share/applications/thothterm.desktop << 'EOF'
[Desktop Entry]
Name=ThothTerm
Comment=The Terminal of Wisdom
Exec=thothterm
Icon=thothterm
Type=Application
Categories=TerminalEmulator;System;
EOF
```

---

## Common Errors

### Error: `error[E0463]: can't find crate for ...`
```bash
rustup update stable
cargo clean && cargo build
```

### Error: `xcb: no DRI3 support`
```bash
# Force Vulkan or OpenGL backend
WGPU_BACKEND=vulkan cargo run --bin thothterm
# OR
WGPU_BACKEND=gl cargo run --bin thothterm
```

### Error: `libssl.so.X: cannot open shared object file`
```bash
sudo apt install libssl-dev
# or set static linking
OPENSSL_STATIC=1 cargo build
```

---

## Development Tips

```bash
# Fast iteration: check without full build
cargo check

# Run with debug logs
THOTHTERM_LOG=debug cargo run --bin thothterm

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Install sccache to speed up rebuilds
cargo install sccache
export RUSTC_WRAPPER=sccache
```
