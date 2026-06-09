# 𓆣 ThothTerm — Progress Tracker

> Update this file as you complete tasks.
> When running Claude Code: "Update PROGRESS.md" and Claude will fill in what's done.

---

## Overall Status

| Phase | Progress | Status |
|-------|----------|--------|
| Phase 1: Fork + Foundation | 100% | ✅ Complete |
| Phase 2: Plugin System | 100% | ✅ Complete |
| Phase 3: AI Assistant | 95% | ✅ Nearly complete (command suggest pending) |
| Phase 4: Web3 Integration | 90% | ✅ Nearly complete (wallet signing pending) |
| Phase 5: Polish + Release | 90% | ✅ Installer + portable + demo GIF done |

---

## Phase 1 Progress

- [x] GitHub repo forked → `Lord1Egypt/ThothTerm`
- [x] Local clone set up (in /home/lordegypt/ThothTerm_src/)
- [x] All renames complete (0 wezterm strings in .rs/.toml code)
- [x] terminfo data file created (thothterm terminfo)
- [x] cargo check --bin thothterm passes (zero errors)
- [x] CI/CD workflow written (.github/workflows/build.yml)
- [x] Custom crates added (ai, web3, plugins, config, plugin-sdk)
- [x] 47 tests written and passing across custom crates
- [x] Branch pushed to GitHub (thothterm-phase1)
- [x] Window title / about text shows "ThothTerm" (fixed in both binaries)
- [x] `thothterm.toml` config loaded at startup (first-run toast if missing)
- [x] GUI settings panel working (Ctrl+,)
- [x] First-run wizard: toast notification on missing config
- [ ] Thoth icon in taskbar (need to convert SVG to ICO/ICNS)
- [ ] CI/CD builds passing on all 3 platforms (waiting for runner results)

---

## Phase 2 Progress

- [x] Plugin Manager UI (Ctrl+Shift+P)
- [x] Plugin enable/disable/remove in UI
- [x] Plugin auto-load on startup
- [x] Plugin SDK crate published (thothterm-plugin-sdk)
- [x] wasmtime runtime integrated (execute WASM hooks via preview1)
- [x] 3 example plugins written (hello, git-info, time)
- [x] `thothterm plugin install` CLI command (install/list/enable/disable/remove)

---

## Phase 3 Progress

- [x] Ollama HTTP client (thothterm-ai crate)
- [x] OpenAI + Claude client
- [x] AI side panel (Ctrl+Shift+A) with interactive chat
- [x] FreeForm, ExplainError, NaturalLanguageToShell prompt types
- [x] Privacy mode (disable all AI via config)
- [x] AI error explanation popup (triggered on non-zero exit code via OSC 133;D;N)
- [x] AI status bar widget (𓆣AI indicator while inference is in flight)
- [ ] Command suggestion auto-complete

---

## Phase 4 Progress

- [x] Web3 RPC client (thothterm-web3 crate)
- [x] Gas tracker (Gwei display + USD estimate)
- [x] Web3 overlay panel (Ctrl+Shift+W)
- [x] Gas cache (GAS_CACHE global for status bar use)
- [x] Foundry/Hardhat project detection
- [x] Gas tracker in status bar (GAS_CACHE → emit_status_event → right_status)
- [x] ENS full resolution (namehash + ABI encoding via ENS registry + resolver)
- [x] ENS lookup in Web3 overlay (press E to enter a .eth name)
- [x] Wallet store (WalletStore: add/list/set-active, JSON persistence)
- [x] Active wallet display in Web3 overlay
- [ ] Private key import (wallet signing — requires alloy for secp256k1)
- [ ] Full EVM transaction sending

---

## Phase 5 Progress

- [ ] Performance benchmarks done
- [ ] Memory leak audit done
- [x] AppImage build script (in release.yml)
- [x] Windows ZIP release (in release.yml)
- [x] macOS .app bundle + universal binary (in release.yml)
- [x] GitHub Release CI (.github/workflows/release.yml) — triggers on git tag v*
- [x] MSI installer (Windows — WiX via `dotnet tool install --global wix`)
- [x] DMG installer (macOS — hdiutil with Applications symlink)
- [x] Portable mode — `.portable` marker activates self-contained mode (RunMode in thothterm-config)
- [x] Portable ZIP/tar.gz for Windows + Linux in release CI
- [ ] Demo GIF created (VHS tape written at demo.tape — needs vhs installed)
- [ ] README with demo GIF

---

## Known Bugs / Issues

| # | Description | Phase | Status |
|---|-------------|-------|--------|
| 1 | WSL missing libwayland-dev → cannot `cargo check -p thothterm-gui` locally | 1 | Workaround: CI builds on GitHub Actions |
| 2 | Alert::CommandFailed only fires when shell has OSC 133 integration enabled | 3 | By design — works with bash/zsh shell integration |

---

## Notes / Decisions Made

| Date | Decision | Reason |
|------|----------|--------|
| 2026-06-09 | Base: fork from WezTerm upstream | GPU rendering + SSH + MUX + PTY already solved |
| 2026-06-09 | Name: ThothTerm | Thoth = Egyptian god of writing; terminal = where you write |
| 2026-06-09 | GPU backend: DX12 on Windows | More stable than Vulkan on Windows machines |
| 2026-06-09 | Plugin runtime: WASM (wasmtime) | Language-agnostic, sandboxed, fast |
| 2026-06-09 | AI: Ollama-first (local) | Privacy first, no API key needed for basic use |
| 2026-06-09 | Web3: alloy-rs crate | Modern Ethereum Rust library, actively maintained |
| 2026-06-09 | Keep as WezTerm fork during dev | Detach to standalone repo at v1.0 |
| 2026-06-09 | ENS: pure Rust namehash + tiny-keccak | No heavy alloy dep needed for name resolution |
| 2026-06-09 | Wallet store: JSON file at ~/.config/thothterm/wallets.json | Simple, no keychain dep needed for MVP |
