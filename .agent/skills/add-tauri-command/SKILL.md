---
name: add-tauri-command
description: >
  Use when adding a new Tauri command that the React frontend needs to call.
  Covers the full checklist: Rust function, registration, capability entry,
  TypeScript wrapper and types, error handling, and testing.
---

# Skill: add-tauri-command

## When to Use

Activate this skill whenever you need to expose new Rust functionality to the frontend via `invoke`.

---

## Checklist

Work through each step in order. Do not skip steps.

---

### 1. Rust Function

Create the handler in `src-tauri/src/` (or a submodule):

```rust
/// Returns the current app config.
#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<Config, String> {
    tkey_core::config::load(&app.path().app_config_dir().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
```

Rules:
- Return `Result<T, String>` or `Result<T, SerializableError>` — never panic.
- Accept `tauri::AppHandle` or `tauri::State<T>` for dependencies — do not use globals.
- Name the command in `snake_case` — this becomes the invoke key.

---

### 2. Register the Command

In `src-tauri/src/lib.rs`, add to `invoke_handler`:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        greet,
        get_config,  // ← add here
    ])
```

**One registration point only.**

---

### 3. Capability Entry (if new permissions needed)

If the command accesses new resources (files, OS APIs), add the minimum required permission to `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "core:default",
    "opener:default",
    "fs:allow-app-local-data-read-recursive"  // ← only if needed
  ]
}
```

Principle: least privilege. Do not add broad permissions.

---

### 4. TypeScript Type

Add the return type to `src/lib/types.ts`:

```ts
export interface Config {
  typingMode: "telex" | "vni" | "pinyin";
  tonePlacement: "modern" | "traditional";
  shortcuts: Shortcut[];
}
```

The TypeScript shape must exactly mirror the Rust struct (same field names in camelCase as Tauri serialises them).

---

### 5. API Wrapper

Add the typed wrapper to `src/lib/api.ts`:

```ts
import type { Config } from "./types";

export async function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}
```

**Never call `invoke` directly in a component.**

---

### 6. Error Handling

- On the Rust side: return `Err(descriptive_message)`.
- On the TS side: wrap calls in `try/catch` and show user-friendly errors (use a shadcn/ui `Toast` or `Alert`).

```ts
try {
  const config = await getConfig();
  setConfig(config);
} catch (err) {
  console.error("Failed to load config:", err);
  // show toast
}
```

---

### 7. Tests

- Rust: unit test the underlying logic (not the Tauri command wrapper itself — that's integration).
- TypeScript: if the wrapper has non-trivial logic, add a unit test.

---

### 8. Quality Gates

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
pnpm tsc --noEmit
# If tauri.conf.json or capabilities changed:
pnpm tauri build --debug
```

---

## Verification Checklist

- [ ] Rust function returns `Result`, never panics
- [ ] Registered in `generate_handler!` (one place)
- [ ] Capability entry added only if new permission needed
- [ ] TypeScript type mirrors Rust struct
- [ ] `api.ts` has the typed wrapper; no `invoke` in components
- [ ] Error shown to user on failure
- [ ] Tests for the underlying logic
- [ ] All quality gates pass
