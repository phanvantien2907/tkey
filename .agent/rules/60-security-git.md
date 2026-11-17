# 60 – Security & Git

## Security

### No Secrets in the Repo
- Never commit API keys, tokens, passwords, or private keys.
- Use environment variables or the OS keychain for sensitive data.
- If a secret is accidentally committed, treat it as compromised immediately.

### IME Process Isolation
- **No network calls** from the IME process (`crates/core` or the platform shell).
  - The IME runs with high trust in the host app — network access is a security risk.
- **No telemetry** of any kind — no usage tracking, no error reporting to external servers.
- Config file is local only; no syncing to cloud without explicit user action.

### Tauri / Web
- CSP must be set (see `20-tauri.md`). No `unsafe-eval`.
- No remote content loaded in the settings window.
- Do not use `shell:allow-execute` without explicit justification and minimal scope.

---

## Git Conventions

### Commit Style: Conventional Commits

Format: `<type>(<scope>): <short description>`

| Type | When |
|------|------|
| `feat` | New feature |
| `fix` | Bug fix |
| `test` | Adding or fixing tests |
| `refactor` | Code change without behavior change |
| `docs` | Documentation only |
| `chore` | Build, CI, config, deps |
| `perf` | Performance improvement |

Examples:
```
feat(core): add VNI tone placement for compound rimes
fix(core): correct ü tone mark placement for 'lv' input
test(core): add table-driven tests for Telex undo
docs(agents): update AGENTS.md with windows-tsf-shell skill
```

### Commit Size
- **Small and focused**: one logical change per commit.
- Do not bundle unrelated refactors with feature work.
- Tests for a feature go in the same commit as the feature (or the commit immediately after).

### What NOT to Commit

Add these to `.gitignore` and never force-push them:

```
target/           # Rust build artifacts
node_modules/     # JS dependencies
dist/             # Vite build output
*.tmp             # Temp config writes
.DS_Store         # macOS metadata
```

### Branch Strategy
- `main`: always buildable and passing all quality gates.
- Feature branches: `feat/<short-name>`, merged via PR.
- Do not push directly to `main` without review.
