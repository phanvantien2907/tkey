# TKey – Agent Guidance Index

**TKey** is a lightweight Input Method Editor (IME) for typing Vietnamese (Telex/VNI) and Chinese Pinyin with tone marks, plus user-defined shortcuts. Settings UI is built with Tauri 2 + React 19 + TypeScript 6.

> This file is the single entry point for all AI agents (Antigravity, Claude Code, Cursor, Codex, etc.).
> Always read the relevant rule and skill files before making changes.

---

## Project Structure

```
tkey/
├── src/                  # React + TypeScript settings UI
├── src-tauri/            # Tauri 2 backend (Rust)
│   ├── src/              # Rust source (lib.rs, commands)
│   ├── capabilities/     # Tauri permission declarations
│   └── tauri.conf.json
├── crates/               # (TO BE CREATED) Pure Rust crates
│   └── core/             # IME engine – no OS dependencies
├── .agent/
│   ├── rules/            # Always-on rules (read first)
│   └── skills/           # On-demand skills (read when task matches)
└── AGENTS.md             # ← you are here
```

**Golden rule: ALL typing logic lives in `crates/core` and ONLY there.**

---

## Always-On Rules

Read these before any task in this repo:

| File | Covers |
|------|--------|
| [`.agent/rules/00-project-overview.md`](.agent/rules/00-project-overview.md) | Architecture, platform priorities, repo map |
| [`.agent/rules/10-rust.md`](.agent/rules/10-rust.md) | Rust coding standards, safety, error handling |
| [`.agent/rules/20-tauri.md`](.agent/rules/20-tauri.md) | Tauri 2 commands, capabilities, security |
| [`.agent/rules/30-react-vite-ts.md`](.agent/rules/30-react-vite-ts.md) | React/Vite/TypeScript, shadcn/ui, API wrapper |
| [`.agent/rules/40-ime-domain.md`](.agent/rules/40-ime-domain.md) | Telex/VNI/Pinyin rules, shortcuts, composition |
| [`.agent/rules/50-quality-gates.md`](.agent/rules/50-quality-gates.md) | Required checks before claiming a task is done |
| [`.agent/rules/60-security-git.md`](.agent/rules/60-security-git.md) | Security, secrets, commit conventions |
| [`.agent/rules/70-communication.md`](.agent/rules/70-communication.md) | Language and communication style with the user |

---

## On-Demand Skills

Activate the relevant skill when your task matches its description:

| Skill | When to use |
|-------|-------------|
| [add-telex-rule](.agent/skills/add-telex-rule/SKILL.md) | Adding or fixing a Vietnamese Telex/VNI typing rule |
| [add-pinyin-tone-rule](.agent/skills/add-pinyin-tone-rule/SKILL.md) | Adding or fixing a Pinyin tone / ü placement rule |
| [add-shortcut-feature](.agent/skills/add-shortcut-feature/SKILL.md) | End-to-end shortcut feature (schema → Tauri → UI) |
| [add-tauri-command](.agent/skills/add-tauri-command/SKILL.md) | Adding a new Tauri command across all layers |
| [core-ffi-boundary](.agent/skills/core-ffi-boundary/SKILL.md) | Exposing `crates/core` as a C ABI static library |
| [macos-imk-shell](.agent/skills/macos-imk-shell/SKILL.md) | Scaffolding the Swift InputMethodKit shell |
| [ime-manual-test-matrix](.agent/skills/ime-manual-test-matrix/SKILL.md) | Running the cross-app manual test checklist |
| [windows-tsf-shell](.agent/skills/windows-tsf-shell/SKILL.md) | *(LATER)* Windows TSF outline – do not implement yet |

---

## Versions (as of project init)

| Technology | Version |
|---|---|
| Rust edition | 2024 |
| Tauri | 2.x |
| React | 19.1 |
| TypeScript | ~6.0.3 |
| Vite | 8.x |
| pnpm lockfile | 9.0 |
| shadcn/ui | (to be installed) |

---

## Key Constraints

- **No `unwrap()`/`expect()` in non-test Rust code** – the IME runs inside a host app and must never panic.
- **No typing logic outside `crates/core`** – not in `src-tauri`, not in `src`.
- **pnpm only** – never `npm install` or `yarn`.
- **Reply to the user in Vietnamese; code/comments/commits in English.**
