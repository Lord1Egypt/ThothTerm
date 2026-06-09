# 𓆣 ThothTerm — Claude Code Context File

> **READ THIS FIRST** — This file is auto-loaded by Claude Code. It contains everything
> you need to understand this project and continue development from any machine.

---

## What is ThothTerm?

ThothTerm is a next-generation GPU-accelerated terminal emulator built by **Mohamed Mounir (Lord1Egypt)**.
Named after Thoth — Egyptian god of writing, wisdom, and knowledge — because a terminal is where you write the future.

**Base**: Fork of [WezTerm](https://github.com/wez/wezterm) (Rust + wgpu)
**Stack**: Rust + wgpu + wasmtime + TOML/JSON config + egui panels
**Goal**: The terminal for the next 10 years — AI-native, Web3-native, WASM-plugin-extensible

---

## Owner

- **GitHub**: github.com/lordegypt  (Lord1Egypt)
- **Email**: akim.221992@gmail.com
- **Style**: Ships fast, writes Rust well (see: ethsmith), Go background too

---

## Current Status

| Phase | Name | Status |
|-------|------|--------|
| Phase 1 | Fork + Rebrand + GUI Settings | 🔴 NOT STARTED |
| Phase 2 | WASM Plugin System | 🔴 NOT STARTED |
| Phase 3 | AI Assistant (Ollama/OpenAI) | 🔴 NOT STARTED |
| Phase 4 | Web3 Integration | 🔴 NOT STARTED |
| Phase 5 | Polish & v1.0 Release | 🔴 NOT STARTED |

**Next immediate action**: See `BUILD_WINDOWS.md` to set up your environment, then `PHASE1.md` to start.

---

## Repo Structure (after fork)

```
ThothTerm/
├── CLAUDE.md               ← you are here
├── PLAN.md                 ← full roadmap
├── ARCHITECTURE.md         ← technical deep-dive
├── BUILD_WINDOWS.md        ← step-by-step Windows build guide
├── BUILD_LINUX.md          ← Linux build guide
├── PHASE1.md               ← Phase 1 tasks (fork + rebrand)
├── PHASE2.md               ← Phase 2 tasks (WASM plugins)
├── PHASE3.md               ← Phase 3 tasks (AI)
├── PHASE4.md               ← Phase 4 tasks (Web3)
├── PHASE5.md               ← Phase 5 tasks (Polish + ship)
├── TIPS_AND_TRICKS.md      ← known issues, gotchas, solutions
├── PLUGIN_SDK.md           ← how to write a ThothTerm plugin
└── src/                    ← source code (after fork)
```

---

## Key Technical Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| Base project | WezTerm fork | GPU rendering + SSH + MUX already done |
| Language | Rust | Performance, WezTerm base, Mohamed knows Rust |
| GPU backend | wgpu (DX12 on Windows, Metal on Mac, Vulkan on Linux) | Cross-platform GPU |
| Plugin runtime | wasmtime (WASM) | Any language can write plugins |
| Config format | TOML + JSON (also keep Lua compat) | User friendly + machine friendly |
| Window system | winit (already in WezTerm) | Best Rust windowing library |
| Font rendering | harfbuzz + fontconfig (already in WezTerm) | Best rendering |
| AI | Ollama (local) + OpenAI/Claude API (cloud) | Privacy first, cloud optional |
| Web3 | alloy-rs crate | Modern Ethereum library for Rust |

---

## ⚠️ Known Gotchas (read before you hit them)

1. **wgpu on Windows** — DX12 backend works great, Vulkan sometimes fails on Intel GPUs.
   Always use `WGPU_BACKEND=dx12` during development on Windows.

2. **WezTerm has 200+ dependencies** — First `cargo build` takes 10-20 minutes. Use `sccache` to cache.

3. **ConPTY on Windows** — WezTerm uses `wincon` feature for ConPTY. Keep this, don't touch it.

4. **wezterm-term crate** — This is the terminal emulator logic (VT parser). It's in the WezTerm monorepo
   as a standalone crate. We keep it as-is in Phase 1, modify in Phase 2+.

5. **Lua config** — WezTerm's Lua config is deep in the code. In Phase 1, ADD JSON/TOML support
   on top. Don't remove Lua yet (breaks existing users who test).

---

## How to Ask Claude for Help

When running Claude Code in this folder on any machine, say:
- "Continue Phase 1" → Claude reads PHASE1.md and continues
- "I hit error X" → Claude reads TIPS_AND_TRICKS.md first, then helps
- "Start the plugin system" → Claude reads PHASE2.md
- "Show me the architecture" → Claude reads ARCHITECTURE.md

---

## Links

- WezTerm source: https://github.com/wez/wezterm
- WezTerm docs: https://wezfurlong.org/wezterm/
- wgpu docs: https://wgpu.rs
- wasmtime: https://wasmtime.dev
- alloy-rs (Web3): https://github.com/alloy-rs/alloy
- Ollama: https://ollama.ai
