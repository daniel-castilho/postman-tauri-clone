# Advanced Scripting & npm Integration — Backlog (Phase 19)

**Companions:** `advanced-scripting-spec.md`
**Epic Goal:** `require('name')` inside sandbox scripts over a bundled, MIT-licensed library registry with an in-app Package Manager.

**MVP Scope:** Stories S1–S6

---

## Story Map

```text
REGISTRY
S1 Vendor lodash/dayjs/crypto-js/uuid bundles + THIRD-PARTY-NOTICES.md
S2 Library registry module (include_str!) + QuickJS smoke tests per bundle

RUNTIME
S3 require() shim + CommonJS wrapper in QuickJsScriptRunner; settings-driven preload

IPC & SETTINGS
S4 Persistence helper (disabled list JSON) + Tauri commands + ts-rs bindings

UI
S5 Script Libraries section in Workspace Settings (thin component)

DOCS
S6 Docs sync (README scripting section, progress tracker) + full gates
```

---

## Stories Breakdown

| ID | Story Title | Priority | Target Modules / Components | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **S1** ✅ | Vendor 4 MIT library bundles + notices file | Must | `src-tauri/assets/script-libs/` | Downloaded from jsDelivr pinned versions; ~151 KB total |
| **S2** ✅ | Registry module + per-bundle smoke tests | Must | `infrastructure/scripting/libraries.rs` | Each source evaluates and exports via CJS wrapper (NIST SHA-256 vector included) |
| **S3** ✅ | `require()` shim in runner setup script | Must | `infrastructure/scripting/quickjs_runner.rs` | Only enabled libs preloaded; unknown-module error lists installed modules |
| **S4** ✅ | Settings persistence + IPC commands + bindings | Must | `presentation/commands.rs`, `application/commands/export_ts_bindings.rs` | `{ disabled: [] }` workspace JSON; re-read per execution |
| **S5** ✅ | Package Manager UI section | Must | `src/components/WorkspaceSettings.tsx` | Thin wrapper: invoke + toggle state only |
| **S6** ✅ | Docs sync + verification gates | Must | `README.md`, `docs/progress.md`, backlog DoD | clippy / cargo test 64 / npm build / boundary grep all green |

---

## Definition of Done (Epic)

- [x] S1–S6 completed and verified.
- [x] A test script using `const _ = require('lodash')` (and one per remaining module) passes in `cargo test`.
- [x] Unknown module error lists installed modules.
- [x] Disabling a library via the new command removes it from the next run's sandbox.
- [x] No changes to Cargo.toml / package.json (Rule 5).
- [x] All gates green: clippy `-D warnings`, `cargo test` (64 passing), `npm run build`, boundary grep 0.

## Deviations from Spec

- **WebCrypto polyfill added** (`SANDBOX_PRELOAD`): uuid@8 throws without `crypto.getRandomValues`; QuickJS ships none. Math.random fallback documented as non-cryptographic — flagged as acceptable for correlation-id use cases.
