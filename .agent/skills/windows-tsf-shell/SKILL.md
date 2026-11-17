---
name: windows-tsf-shell
description: >
  STUB – LATER. High-level outline of the Windows Text Services Framework (TSF)
  integration. Do NOT implement this yet. Read to understand the planned approach.
---

# Skill: windows-tsf-shell

> ⚠️ **STATUS: LATER – DO NOT IMPLEMENT**
>
> This is an outline only. When Windows support becomes a priority,
> this stub will be expanded into a full implementation skill.
> Until then, ignore this file when working on macOS or core tasks.

---

## Planned Approach (Outline)

### Technology

- Implement the TSF text service entirely in **Rust** using the `windows` crate (`windows-rs`).
- The text service calls `crates/core` directly (Rust-to-Rust, no C ABI needed).
- Produces a DLL (`tkey_tsf.dll`) registered as a COM in-process server.

### Key TSF Interfaces to Implement

| Interface | Purpose |
|-----------|---------|
| `ITfTextInputProcessor` | Entry point, activated/deactivated by TSF |
| `ITfKeyEventSink` | Intercept keystrokes |
| `ITfCompositionSink` | Handle preedit (composition) lifecycle |
| `ITfEditSession` | Write composed text to the document |
| `ITfDisplayAttributeProvider` | Underline preedit text |

### Registration

- Register the DLL as a COM server in the Windows Registry.
- Register the input method with `ITfInputProcessorProfiles`.
- Installer must run `regsvr32 tkey_tsf.dll` or use the Windows API directly.

### Composition Model

- Match the same `crates/core` composition model (one word, commit on Space/Enter/punct, Esc cancels).
- Preedit via TSF composition (not fake-backspace).

### Pitfalls to Address (when implementing)

- **Thread safety**: TSF callbacks may come from multiple threads — `crates/core` must be `Send + Sync` or wrapped in a `Mutex`.
- **COM lifetimes**: reference counting via `windows-rs` `IUnknown` — be very careful with `AddRef`/`Release`.
- **Secure input**: TSF has similar secure-input restrictions to IMK on macOS.
- **Elevated processes**: UAC-elevated apps require the input method to be installed at the system level.

### References (for when this is implemented)

- Microsoft TSF docs: https://learn.microsoft.com/en-us/windows/win32/tsf/text-services-framework
- `windows-rs` crate: https://github.com/microsoft/windows-rs
- Example Rust TSF: search GitHub for "rust tsf input method"

---

*This stub will be replaced with a full SKILL.md when Windows platform support is prioritised.*
