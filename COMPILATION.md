# 𓆣 ThothTerm — Compilation Guide (WSL + GitHub Actions)

---

## WSL Compilation — Honest Truth

| Target | From WSL2 | Result |
|--------|-----------|--------|
| Linux x86_64 binary | ✅ Native | Perfect — develop here |
| Windows .exe (MinGW) | ⚠️ Possible | DX12/wgpu breaks — NOT usable for us |
| Windows .exe (MSVC) | ❌ Not from WSL | Needs Windows or GitHub Actions |
| macOS .dmg | ❌ Impossible | Apple SDK license blocks redistribution |

**Bottom line**: Use WSL for Linux development and testing.
Use GitHub Actions CI for Windows + macOS release builds.
This is how 100% of serious cross-platform Rust projects work.

---

## Development Workflow

```
Your WSL (daily dev):
  → write code
  → cargo check (fast, no binary)
  → cargo test (run tests)
  → cargo run --bin thothterm (test Linux)
  → git push

GitHub Actions (automatic on every push):
  → Windows runner builds .exe
  → Linux runner builds AppImage
  → macOS runner builds .dmg
  → Uploads artifacts to GitHub Release
```

---

## WSL: Linux Build (your day-to-day)

### Install System Dependencies (one-time)

```bash
sudo apt update && sudo apt install -y \
  build-essential cmake pkg-config \
  libssl-dev \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libfontconfig1-dev libfreetype6-dev libharfbuzz-dev \
  libssh2-1-dev libzstd-dev \
  python3 git curl \
  mold          # faster linker — 10x speedup on linking
```

### Add Fast Linker Config (one-time, speeds up builds a lot)

```bash
mkdir -p /home/lordegypt/ThothTerm/.cargo
cat > /home/lordegypt/ThothTerm/.cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# sccache for build caching across cargo invocations
# [build]
# rustc-wrapper = "sccache"   # uncomment after: cargo install sccache
EOF
```

### Build Commands

```bash
cd /home/lordegypt/ThothTerm

# Type check only (fastest — use while writing code)
cargo check

# Check all crates + clippy linting
cargo check --workspace && cargo clippy --workspace -- -D warnings

# Build debug (for running/testing)
cargo build --bin thothterm

# Build release (for distribution)
cargo build --release --bin thothterm

# Run
cargo run --bin thothterm

# Run with debug logs
THOTHTERM_LOG=debug cargo run --bin thothterm
```

---

## GitHub Actions: All 3 Platforms

### Setup (one-time on GitHub)

1. Push ThothTerm to `github.com/lordegypt/ThothTerm`
2. GitHub Actions runs automatically on every push
3. Binaries appear under **Actions → latest run → Artifacts**
4. Tagged releases (`git tag v1.0.0 && git push --tags`) → full release with all 3 binaries

### CI Configuration

File: `.github/workflows/release.yml`

```yaml
name: Build & Release ThothTerm

on:
  push:
    branches: [main]
  release:
    types: [created]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ─────────────────────────────────
  # Linux Build (Ubuntu)
  # ─────────────────────────────────
  build-linux:
    name: Build Linux
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt update && sudo apt install -y \
            build-essential cmake pkg-config \
            libssl-dev libxcb1-dev libxcb-render0-dev \
            libxcb-shape0-dev libxcb-xfixes0-dev \
            libxkbcommon-dev libfontconfig1-dev \
            libfreetype6-dev libharfbuzz-dev libssh2-1-dev

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache Cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Run tests
        run: cargo test --workspace

      - name: Build release binary
        run: cargo build --release --bin thothterm

      - name: Create AppImage
        run: |
          # Install appimagetool
          wget -O appimagetool.AppImage \
            https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
          chmod +x appimagetool.AppImage

          mkdir -p AppDir/usr/bin
          cp target/release/thothterm AppDir/usr/bin/
          cp assets/thothterm.png AppDir/thothterm.png
          cat > AppDir/thothterm.desktop << 'DESKTOP'
          [Desktop Entry]
          Name=ThothTerm
          Exec=thothterm
          Icon=thothterm
          Type=Application
          Categories=TerminalEmulator;
          DESKTOP
          ./appimagetool.AppImage AppDir ThothTerm-linux.AppImage

      - name: Upload Linux artifact
        uses: actions/upload-artifact@v4
        with:
          name: thothterm-linux
          path: ThothTerm-linux.AppImage

  # ─────────────────────────────────
  # Windows Build (MSVC — correct GPU)
  # ─────────────────────────────────
  build-windows:
    name: Build Windows
    runs-on: windows-latest
    env:
      WGPU_BACKEND: dx12
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable (MSVC)
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: Cache Cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --workspace

      - name: Build release binary
        run: cargo build --release --bin thothterm

      - name: Build MSI installer
        run: |
          cargo install cargo-wix --version 0.3.7
          cargo wix build --nocapture

      - name: Upload Windows artifact
        uses: actions/upload-artifact@v4
        with:
          name: thothterm-windows
          path: |
            target/release/thothterm.exe
            target/wix/*.msi

  # ─────────────────────────────────
  # macOS Build (Metal GPU)
  # ─────────────────────────────────
  build-macos:
    name: Build macOS
    runs-on: macos-14     # Apple Silicon runner (M1)
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin

      - name: Cache Cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --workspace

      - name: Build universal binary (Intel + Apple Silicon)
        run: |
          cargo build --release --target aarch64-apple-darwin --bin thothterm
          cargo build --release --target x86_64-apple-darwin --bin thothterm
          lipo -create \
            target/aarch64-apple-darwin/release/thothterm \
            target/x86_64-apple-darwin/release/thothterm \
            -output thothterm-universal

      - name: Create .app bundle
        run: |
          mkdir -p ThothTerm.app/Contents/MacOS
          mkdir -p ThothTerm.app/Contents/Resources
          cp thothterm-universal ThothTerm.app/Contents/MacOS/thothterm
          cp assets/thothterm.icns ThothTerm.app/Contents/Resources/ || true
          cat > ThothTerm.app/Contents/Info.plist << 'PLIST'
          <?xml version="1.0" encoding="UTF-8"?>
          <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
          <plist version="1.0">
          <dict>
            <key>CFBundleName</key><string>ThothTerm</string>
            <key>CFBundleIdentifier</key><string>io.github.lordegypt.thothterm</string>
            <key>CFBundleVersion</key><string>0.1.0</string>
            <key>CFBundleExecutable</key><string>thothterm</string>
            <key>LSMinimumSystemVersion</key><string>13.0</string>
          </dict>
          </plist>
          PLIST
          hdiutil create -volname ThothTerm -srcfolder ThothTerm.app -ov -format UDZO ThothTerm-macos.dmg

      - name: Upload macOS artifact
        uses: actions/upload-artifact@v4
        with:
          name: thothterm-macos
          path: ThothTerm-macos.dmg

  # ─────────────────────────────────
  # Create GitHub Release (on tag push)
  # ─────────────────────────────────
  release:
    name: Create Release
    runs-on: ubuntu-latest
    needs: [build-linux, build-windows, build-macos]
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            thothterm-linux/ThothTerm-linux.AppImage
            thothterm-windows/thothterm.exe
            thothterm-windows/*.msi
            thothterm-macos/ThothTerm-macos.dmg
          generate_release_notes: true
```

---

## How to Release v1.0.0

```bash
# From your WSL:
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions automatically:
# 1. Builds all 3 platforms
# 2. Creates GitHub Release
# 3. Attaches all binaries
# Done in ~15 minutes
```

---

## WSL GPU Limitation for Testing

Since WSL doesn't have a real GPU context, running the full GUI in WSL may show:
```
wgpu: no suitable adapter found
```

**For WSL testing**: Use headless mode or test individual crates without the GUI.
```bash
# Test just the AI crate (no GPU needed)
cargo test -p thothterm-ai

# Test just config parsing
cargo test -p thothterm-config

# Test Web3 crate
cargo test -p thothterm-web3

# Run full GUI — needs real Linux with GPU or Xwayland
DISPLAY=:0 cargo run --bin thothterm
```

The GUI renders fine on a real Linux desktop or Windows (via native build).
