---
name: core-ffi-boundary
description: >
  Use when exposing crates/core as a C ABI static library for consumption by
  platform shells (Swift InputMethodKit on macOS, TSF on Windows).
  Covers extern "C" API design, opaque handles, UTF-8 string ownership,
  panic safety, header generation with cbindgen, and Swift FFI consumption.
---

# Skill: core-ffi-boundary

## When to Use

Activate this skill when:
- Creating the C ABI layer that the Swift macOS shell will call into
- Adding a new function to the stable `extern "C"` API
- Setting up cbindgen for header generation

---

## Architecture

```
Swift InputMethodKit shell
        │
        │ calls via C ABI (dlopen / static link)
        ▼
┌─────────────────────────────────┐
│  crates/core-ffi  (new crate)  │  ← FFI boundary (unsafe allowed here)
│  - extern "C" functions        │
│  - catch_unwind on every call  │
│  - UTF-8 string helpers        │
└───────────┬─────────────────────┘
            │ depends on
┌───────────▼─────────────────────┐
│  crates/core  (pure Rust)      │  ← no unsafe, no C ABI knowledge
└─────────────────────────────────┘
```

---

## Step 1 – Create `crates/core-ffi`

```toml
# crates/core-ffi/Cargo.toml
[package]
name = "tkey-core-ffi"
edition = "2024"

[lib]
crate-type = ["staticlib", "cdylib"]

[dependencies]
tkey-core = { path = "../core" }
```

Add to workspace `Cargo.toml`:
```toml
members = ["crates/core", "crates/core-ffi", "src-tauri"]
```

---

## Step 2 – Opaque Handle Pattern

```rust
// crates/core-ffi/src/lib.rs
use std::ffi::{c_char, CStr, CString};
use tkey_core::Engine;

pub struct TKeyEngine(Engine);

#[no_mangle]
pub extern "C" fn tkey_engine_create() -> *mut TKeyEngine {
    // SAFETY: Box::into_raw gives ownership to the C caller.
    // Caller must free with tkey_engine_destroy.
    Box::into_raw(Box::new(TKeyEngine(Engine::new())))
}

#[no_mangle]
pub extern "C" fn tkey_engine_destroy(engine: *mut TKeyEngine) {
    if engine.is_null() { return; }
    // SAFETY: pointer came from tkey_engine_create and caller passes ownership back.
    unsafe { drop(Box::from_raw(engine)); }
}
```

---

## Step 3 – UTF-8 String Rules

- **Input strings**: accept `*const c_char`, validate with `CStr::from_ptr`, convert to `&str` with `.to_str()`.
- **Output strings**: return `*mut c_char` allocated with `CString::new(...).unwrap().into_raw()`.
- **Caller frees output strings** using a companion `tkey_free_string(ptr: *mut c_char)` function.
- Never return a pointer to a Rust-owned `String` — it will be freed by Rust's allocator.

```rust
#[no_mangle]
pub extern "C" fn tkey_free_string(s: *mut c_char) {
    if s.is_null() { return; }
    // SAFETY: s was allocated by CString::into_raw in this crate.
    unsafe { drop(CString::from_raw(s)); }
}
```

---

## Step 4 – No Panics Across FFI (`catch_unwind`)

Rust panics must never unwind across the FFI boundary — it is undefined behaviour.

```rust
use std::panic::catch_unwind;

#[no_mangle]
pub extern "C" fn tkey_process_key(
    engine: *mut TKeyEngine,
    key: c_char,
) -> *mut c_char {
    let result = catch_unwind(|| {
        // SAFETY: pointer lifetime is managed by caller (see handle pattern).
        let engine = unsafe { &mut (*engine).0 };
        engine.process_key(key as u8 as char)
    });
    match result {
        Ok(output) => CString::new(output).unwrap_or_default().into_raw(),
        Err(_) => std::ptr::null_mut(), // panic caught, return null
    }
}
```

---

## Step 5 – Header Generation with cbindgen

Add to `crates/core-ffi/build.rs`:

```rust
fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("cbindgen failed")
        .write_to_file("include/tkey_core.h");
}
```

Add to `Cargo.toml`:
```toml
[build-dependencies]
cbindgen = "0.27"
```

Commit the generated `include/tkey_core.h` — it is the stable contract.

---

## Step 6 – Swift Consumption

```swift
// In the Swift InputMethodKit target
import Foundation

// Load the static lib (linked via Xcode build settings)
// Use the generated header via a bridging header or module map

func processKey(_ char: Character) -> String? {
    guard let engine = tkey_engine_create() else { return nil }
    defer { tkey_engine_destroy(engine) }

    let keyStr = String(char)
    return keyStr.withCString { cStr in
        guard let result = tkey_process_key(engine, cStr.pointee) else { return nil }
        defer { tkey_free_string(result) }
        return String(cString: result)
    }
}
```

---

## Verification Checklist

- [ ] `catch_unwind` wraps every `extern "C"` function body
- [ ] Every `unsafe` block has a `// SAFETY:` comment
- [ ] `tkey_free_string` exists and is documented for callers
- [ ] cbindgen generates `include/tkey_core.h` without warnings
- [ ] Swift test calls create/process/destroy without memory leaks (Instruments: Leaks)
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
