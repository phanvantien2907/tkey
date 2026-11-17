---
name: add-shortcut-feature
description: >
  Use for any end-to-end change involving user-defined shortcuts (trigger → output mappings).
  Covers the full stack: core schema, validation, Tauri command, TypeScript wrapper, React UI,
  and tests at each layer.
---

# Skill: add-shortcut-feature

## When to Use

Activate this skill when:
- Adding CRUD operations for shortcuts (create, read, update, delete)
- Changing the shortcut expansion logic (commit key, mid-word guard, escape mechanism)
- Updating the TOML/JSON schema for shortcut entries
- Building the React settings UI for shortcut management

---

## Layers to Touch (in order)

```
crates/core       → schema + validation + expansion logic
src-tauri/src     → Tauri command(s)
src-tauri/capabilities → permission entry if new fs access needed
src/lib/types.ts  → TypeScript mirror types
src/lib/api.ts    → typed invoke wrapper
src/components/   → React UI (shadcn/ui components)
```

---

## Workflow

### Step 1 – Core schema (`crates/core`)

Define or update the shortcut type:

```rust
// crates/core/src/shortcut.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub trigger: String,   // e.g. "nh"
    pub output: String,    // e.g. "níhǎo"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShortcutConfig {
    pub shortcuts: Vec<Shortcut>,
}
```

Add validation:
- `trigger` must be non-empty, ASCII only, no whitespace.
- `output` must be valid UTF-8.
- No duplicate triggers.

Write tests for validation edge cases.

### Step 2 – Expansion logic (`crates/core`)

Write failing tests first:

```rust
#[test]
fn shortcut_expands_on_space() {
    let config = ShortcutConfig { shortcuts: vec![
        Shortcut { trigger: "nh".into(), output: "níhǎo".into() },
    ]};
    assert_eq!(expand_shortcut("nh", &config), Some("níhǎo"));
    assert_eq!(expand_shortcut("nha", &config), None); // mid-word guard
    assert_eq!(expand_shortcut("nk", &config), None);  // no match
}
```

### Step 3 – Tauri commands (`src-tauri/src/`)

Follow `.agent/rules/20-tauri.md` and `.agent/skills/add-tauri-command/SKILL.md`:

```rust
#[tauri::command]
fn get_shortcuts(app: tauri::AppHandle) -> Result<Vec<Shortcut>, String> { ... }

#[tauri::command]
fn save_shortcuts(app: tauri::AppHandle, shortcuts: Vec<Shortcut>) -> Result<(), String> { ... }
```

Register both in `tauri::Builder::invoke_handler`.

### Step 4 – TypeScript types (`src/lib/types.ts`)

Mirror the Rust structs exactly:

```ts
export interface Shortcut {
  trigger: string;
  output: string;
}
```

### Step 5 – API wrapper (`src/lib/api.ts`)

```ts
import { invoke } from "@tauri-apps/api/core";
import type { Shortcut } from "./types";

export async function getShortcuts(): Promise<Shortcut[]> {
  return invoke<Shortcut[]>("get_shortcuts");
}

export async function saveShortcuts(shortcuts: Shortcut[]): Promise<void> {
  return invoke<void>("save_shortcuts", { shortcuts });
}
```

### Step 6 – React UI (`src/components/`)

Use shadcn/ui components:
- `Table` for listing shortcuts
- `Input` for trigger and output fields
- `Button` for add/remove/save actions
- `Form` with validation feedback

Accessibility: every input has a `<label>`.

### Step 7 – Quality gates

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
pnpm tsc --noEmit
```

---

## Verification Checklist

- [ ] Core validation tests pass
- [ ] Expansion logic tests: match, mid-word guard, escape
- [ ] Tauri commands registered and returning correct types
- [ ] TypeScript types match Rust structs
- [ ] `api.ts` is the only place `invoke` is called
- [ ] React UI uses shadcn/ui, all inputs labelled
- [ ] All quality gates pass
