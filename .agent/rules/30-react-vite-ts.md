# 30 – React, Vite & TypeScript

## TypeScript

- `strict: true` is already set in `tsconfig.json` — maintain it, never weaken it.
- **No `any`** — use proper types, generics, or `unknown` with narrowing.
- **No non-null assertion `!`** without an inline comment explaining why it is safe:
  ```ts
  // ✅ OK with explanation
  const el = document.getElementById("root")!; // guaranteed by index.html

  // ❌ Not OK
  const val = maybeNull!;
  ```
- Prefer `type` aliases for data shapes and `interface` for extensible contracts.
- Run `pnpm tsc --noEmit` before every commit.

---

## React

- Use **function components** and hooks — no class components.
- Keep components **small and focused**: one responsibility per component.
- Keep state **local** unless it must be shared; lift state only when necessary.
- Avoid prop-drilling deeper than 2 levels — use React context or a small state module.
- Always declare dependency arrays correctly in `useEffect`, `useMemo`, `useCallback`.

---

## shadcn/ui

- **shadcn/ui is the designated UI component library** for TKey's settings UI.
- Install components with `pnpm dlx shadcn@latest add <component>` — components live in `src/components/ui/`.
- Do **not** add another UI kit alongside shadcn/ui (Ant Design, MUI, Chakra, etc.).
- When a shadcn/ui component covers the need, use it. Build custom components only when shadcn/ui has no equivalent.
- Tailwind CSS is used by shadcn/ui — keep utility classes organized; extract repeated patterns into component files.
- Keep the shadcn/ui theme consistent: edit `src/index.css` variables rather than hard-coding colors.

---

## Tauri API Wrapper

- **All `invoke` calls go through `src/lib/api.ts`** — never call `invoke` directly in components.
- `api.ts` exports typed async functions mirroring the Rust command signatures:
  ```ts
  // src/lib/api.ts
  import { invoke } from "@tauri-apps/api/core";
  import type { Config } from "./types";

  export async function getConfig(): Promise<Config> {
    return invoke<Config>("get_config");
  }
  ```
- TypeScript types in `src/lib/types.ts` must mirror the Rust structs (keep them in sync manually or via `ts-rs`).

---

## Package Manager

- **pnpm only** — never run `npm install` or `yarn`.
- Add dependencies with `pnpm add <pkg>` and dev deps with `pnpm add -D <pkg>`.
- Keep `pnpm-lock.yaml` committed and up to date.

---

## Internationalization

- Support **Vietnamese** and **English** UI strings via a simple i18n map (no heavy i18n library needed initially):
  ```ts
  const i18n = { vi: { save: "Lưu" }, en: { save: "Save" } };
  ```
- Never hard-code display strings in components — reference the i18n map.

---

## Text Rendering

- **UTF-8 everywhere** — all string handling must be Unicode-safe.
- Use a system font stack with **CJK fallback** for rendering Vietnamese and Chinese:
  ```css
  font-family: system-ui, "Segoe UI", sans-serif, "Noto Sans CJK SC", "PingFang SC";
  ```
- Test rendered output with Vietnamese tone marks and Chinese characters.

---

## Accessibility

- Every form control has an associated `<label>`.
- Interactive elements are keyboard-navigable (Tab order, `aria-*` where needed).
- shadcn/ui components are ARIA-compliant by default — do not remove those attributes.
