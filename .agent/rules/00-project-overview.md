# 00 – Project Overview

## What is TKey?

TKey is a lightweight Input Method Editor (IME) for:
- **Vietnamese**: Telex and VNI schemes with correct tone marks (`hoà`, `hòa` per config)
- **Chinese Pinyin**: tone marks via digits (`ni3hao3` → `nǐhǎo`)
- **User shortcuts**: custom trigger → output expansions (e.g. `nh` + Space → `níhǎo`)

**Priority platforms**: macOS first, Windows second. Linux/Android/iOS: later.

---

## Architecture

```
┌─────────────────────────────────────────┐
│  macOS: Swift InputMethodKit shell      │  ← platform shell (future)
│  Windows: TSF in Rust shell             │
└───────────────┬─────────────────────────┘
                │ C ABI (cbindgen / UniFFI)
┌───────────────▼─────────────────────────┐
│  crates/core  (pure Rust, no OS deps)   │  ← ALL typing logic here
│  - Telex/VNI engine                     │
│  - Pinyin tone engine                   │
│  - Shortcut manager                     │
│  - Config schema + validation           │
└───────────────┬─────────────────────────┘
                │ depends on (config schema only)
┌───────────────▼─────────────────────────┐
│  src-tauri  (Tauri 2, Rust)             │  ← settings UI backend
│  - Tauri commands (read/write config)   │
│  - File-watch for config changes        │
└───────────────┬─────────────────────────┘
                │ invoke / events
┌───────────────▼─────────────────────────┐
│  src/  (React 19 + Vite 8 + TS 6)      │  ← settings UI only
│  - shadcn/ui components                 │
│  - All Tauri calls via src/lib/api.ts   │
└─────────────────────────────────────────┘
```

**IME ↔ settings communication**: shared config file (TOML/JSON) in the OS user config dir, watched via file events. No complex IPC.

---

## Golden Rule

> **ALL typing logic lives in `crates/core` and ONLY there.**
>
> - `src-tauri` may read config schema/types from `crates/core`, but contains no typing logic.
> - `src/` (React) contains **no** typing logic whatsoever.
> - Platform shells (Swift, TSF) call `crates/core` through the C ABI.

---

## Repo Map

```
tkey/
├── AGENTS.md                 # agent guidance index
├── Cargo.toml                # (FUTURE) workspace root: members = ["crates/core", "src-tauri"]
├── package.json              # pnpm project
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json             # strict: true
├── tsconfig.node.json
├── index.html
├── src/                      # React settings UI
│   ├── lib/api.ts            # (to create) single Tauri call wrapper
│   ├── components/           # shadcn/ui + custom components
│   └── main.tsx
├── src-tauri/                # Tauri 2 backend
│   ├── Cargo.toml            # edition 2024
│   ├── capabilities/
│   │   └── default.json      # core:default + opener:default
│   ├── src/
│   │   └── lib.rs
│   └── tauri.conf.json
├── crates/                   # (TO CREATE when implementing core)
│   └── core/
│       ├── Cargo.toml        # #![forbid(unsafe_code)], thiserror, no anyhow
│       └── src/
└── .agent/
    ├── rules/
    └── skills/
```

---

## Product Constraints

- **Extremely lightweight**: minimal RAM, instant startup.
- **No crashes**: the IME runs in the host app process or as a system service — a panic is a host-app crash.
- **Works everywhere**: browsers, WPS Office, Gmail, Facebook, terminal.
- **Composition via official OS IME API** (preedit/marked text) — never fake-backspace hooks.
- **No candidate window** for Chinese characters unless explicitly requested.
- **No telemetry, no network calls** from the IME process.
