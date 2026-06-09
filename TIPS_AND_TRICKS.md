# 𓆣 ThothTerm — Tips, Tricks, and Known Issues

> This file is your survival guide. Read it when you hit a wall.

---

## #1 — Windows GPU Backend (Most Common Issue)

**Problem**: `wgpu: no suitable adapter found` or blank window on Windows

**Fix**:
```powershell
$env:WGPU_BACKEND = "dx12"
cargo run --bin thothterm
```

**Why it happens**: wgpu auto-detects Vulkan first, which fails on many Intel/AMD GPUs.
DX12 is available on all Windows 10+ machines.

**Make it permanent**: Add `WGPU_BACKEND=dx12` to your system environment variables.

---

## #2 — Slow First Build

**Problem**: `cargo build` takes 15-20 minutes on first run.

**Why**: The project has ~200 dependencies, all compiled from source the first time.

**Solutions**:
```bash
# Install sccache (build cache across projects)
cargo install sccache
export RUSTC_WRAPPER=sccache   # Linux/Mac
$env:RUSTC_WRAPPER="sccache"   # Windows

# Use faster linker on Windows
# Add to .cargo/config.toml:
[target.x86_64-pc-windows-msvc]
linker = "rust-lld"

# Use mold linker on Linux (10x faster linking)
sudo apt install mold
# Add to .cargo/config.toml:
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

---

## #3 — Renaming Gotchas After Fork

After bulk-renaming `wezterm` → `thothterm`, some things might break:

1. **Cargo workspace members** — check `Cargo.toml` at root, all `members = [...]` paths
2. **Feature flags** — some features are named `wezterm-*`, rename carefully
3. **Environment variables** — `WEZTERM_CONFIG_FILE` → `THOTHTERM_CONFIG_FILE`
4. **Registry keys on Windows** — search `HKCU\Software\WezTerm` and rename

Quick check after rename:
```bash
grep -r "wezterm" --include="*.rs" --include="*.toml" . | grep -v "target/" | grep -v ".git/"
# Should show zero results (or only comments explaining the upstream)
```

---

## #4 — WASM Plugin Sandbox Permissions

When writing or testing plugins, the WASM sandbox denies network access by default.

To allow a plugin to make HTTP calls, add to `plugin.toml`:
```toml
permissions = ["network"]
```

For file system access:
```toml
permissions = ["fs:read:/home/user/projects"]
```

---

## #5 — Font Rendering Issues

**Problem**: Fonts look blurry or pixelated.

**Fix** (in `thothterm.toml`):
```toml
[appearance]
font_family = "JetBrains Mono"
font_size = 14.0
font_antialias = "subpixel"     # options: none, grayscale, subpixel
font_hinting = "full"           # options: none, slight, medium, full
```

**On Windows with ClearType**: `font_antialias = "subpixel"` looks best.
**On HiDPI screens**: `font_antialias = "grayscale"` looks better.

---

## #6 — SSH Connection Drops

**Problem**: SSH sessions disconnect after idle period.

**Fix** (in `thothterm.toml`):
```toml
[ssh.keepalive]
enabled = true
interval_seconds = 30
max_count = 3
```

Also add to your `~/.ssh/config`:
```
Host *
  ServerAliveInterval 30
  ServerAliveCountMax 3
```

---

## #7 — Ollama Not Connecting (AI Plugin)

**Problem**: AI features show "Cannot connect to Ollama"

**Checklist**:
```bash
# 1. Is Ollama running?
ollama serve

# 2. Is the model downloaded?
ollama pull llama3.2

# 3. Test manually
curl http://localhost:11434/api/generate -d '{
  "model": "llama3.2",
  "prompt": "Hello"
}'
```

**In config**:
```toml
[ai]
enabled = true
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama3.2"
```

---

## #8 — Cargo Check vs Cargo Build

During development, use `cargo check` constantly — it only checks types/borrow checker, takes ~5x less time than a full build.

```bash
# Fast type check (no binary produced)
cargo check

# Check with all features
cargo check --all-features

# Only when ready to run:
cargo build
```

---

## #9 — Debugging ThothTerm Itself

```bash
# Enable all logs
THOTHTERM_LOG=trace cargo run --bin thothterm 2>&1 | tee thothterm.log

# Enable specific module logs
THOTHTERM_LOG=thothterm_core::pty=debug cargo run --bin thothterm

# On Windows:
$env:THOTHTERM_LOG = "trace"
cargo run --bin thothterm 2>&1 | Tee-Object thothterm.log
```

---

## #10 — Keeping Up With Upstream

Every few weeks, merge changes from the original upstream source:

```bash
git fetch upstream
git merge upstream/main

# Resolve any conflicts (usually in renamed files)
# Test that everything still builds:
cargo build

# Push update
git push origin main
```

---

## #11 — ConPTY on Windows (Important!)

ThothTerm uses Windows ConPTY for terminal emulation on Windows.
**Do NOT remove or disable the `wincon` feature flag** in `Cargo.toml`.

If you see garbled output on Windows PowerShell, it's usually a ConPTY issue:
```powershell
# Test with explicit ConPTY mode
$env:THOTHTERM_WIN_CONPTY = "1"
.\thothterm.exe
```

---

## #12 — Cross-compiling for Windows from Linux

```bash
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW linker
sudo apt install mingw-w64

# Build for Windows
cargo build --target x86_64-pc-windows-gnu --release
```

> ⚠️ Use `x86_64-pc-windows-gnu` for cross-compilation, but prefer native MSVC (`x86_64-pc-windows-msvc`) when building ON Windows for best compatibility.

---

## #13 — Creating the Demo GIF

Every project by Mohamed needs an animated terminal GIF in the README (see memory).

```bash
# Install VHS (preferred)
cargo install vhs
# OR: https://github.com/charmbracelet/vhs

# Create demo script: demo.tape
cat > demo.tape << 'EOF'
Output demo.gif
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"

Type "thothterm"
Sleep 1s
# ... add demo commands
EOF

vhs demo.tape
```

---

## Quick Reference — Important File Locations

| File | Location |
|------|----------|
| User config | `~/.config/thothterm/thothterm.toml` (Linux/Mac) |
| User config | `%APPDATA%\ThothTerm\thothterm.toml` (Windows) |
| Plugins dir | `~/.config/thothterm/plugins/` |
| Logs | `~/.config/thothterm/logs/` |
| Session data | `~/.local/share/thothterm/` |

---

## When Stuck: How to Ask Claude for Help

Open Claude Code in the ThothTerm directory and say:
- "I'm getting error: [paste error]" → Claude reads TIPS_AND_TRICKS.md + helps fix
- "How do I implement [feature]?" → Claude reads ARCHITECTURE.md + guides you
- "Continue Phase N" → Claude reads the phase file and continues tasks
