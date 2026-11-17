# 20 – Tauri 2 Conventions

## Commands

- Expose Rust functions to the frontend using `#[tauri::command]`.
- **Register all commands in one place** (`tauri::Builder::invoke_handler`), never scattered.
- Return `Result<T, String>` or `Result<T, SerializableError>` — never return raw panics.

```rust
// ✅ DO
#[tauri::command]
fn get_config(app: tauri::AppHandle) -> Result<Config, String> {
    load_config(&app).map_err(|e| e.to_string())
}

// ❌ DON'T
#[tauri::command]
fn get_config() -> Config {
    load_config().unwrap() // panics, no error info to frontend
}
```

---

## Capabilities & Permissions (Least Privilege)

- `src-tauri/capabilities/*.json` must list **only permissions that are actually used**.
- Current baseline: `core:default` + `opener:default`.
- **Filesystem access**: restrict to the TKey config directory only. Do not use `fs:allow-home-dir` or `fs:allow-all`.
  ```json
  // Example: only allow reading/writing the app config dir
  "fs:allow-app-local-data-read-recursive",
  "fs:allow-app-local-data-write"
  ```
- **No shell permissions** (`shell:allow-execute`) unless absolutely required and justified.

---

## Content Security Policy

> ⚠️ **Current repo state**: `tauri.conf.json` has `"csp": null` (disabled).
> This must be fixed before any production build.

- Set a strict CSP that allows **no remote content** and **no `eval`**:
  ```json
  "security": {
    "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:"
  }
  ```
- Never use `unsafe-eval` or `unsafe-inline` for scripts.

---

## Config Read / Write

- All config I/O goes through `crates/core`'s schema types and validation.
- **Atomic writes**: write to a temp file first, then `fs::rename()` to the target.
  ```rust
  let tmp = config_path.with_extension("tmp");
  fs::write(&tmp, serialized)?;
  fs::rename(&tmp, &config_path)?;
  ```
- Never write partial/corrupt config directly to the target file.

---

## Settings Window Lifecycle

- The settings window is **lazy**: only open when the user requests it.
- **Closing the window must not affect the running IME** — the IME process is separate.
- Keep the window small and fast to load (small JS bundle, no heavy libs beyond what's needed).
- Prefer `tauri::window::WindowBuilder` lazy creation over always keeping a hidden window alive.

---

## Tauri Plugin Usage

- When adding a new plugin, add its permission to `capabilities/*.json` and document why.
- Prefer official Tauri plugins (`@tauri-apps/plugin-*`) over third-party wrappers.
