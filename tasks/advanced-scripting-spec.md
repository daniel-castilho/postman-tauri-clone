# Advanced Scripting & npm Integration — Spec (Phase 19)

**Companions:** `advanced-scripting-backlog.md`
**Epic Goal:** Let scripts import well-known JavaScript libraries through `require('name')` inside the QuickJS sandbox, with an in-app Package Manager to enable/disable them per workspace.

**Version basis:** `tyny-pulse 0.1.0`
**Approved approach:** bundling of four MIT libraries (user decision, Rule 5 review): **lodash**, **dayjs**, **crypto-js**, **uuid**.

---

## 1. Problem Statement

Scripts currently run against bare shims (`pm`, `console`, `expect`). Common automation needs — deep object cloning, date formatting, hashing/signing payloads, correlation ids — force users to hand-roll utilities. There is no module system in the sandbox.

---

## 2. Solution Overview

Postman-style curated runtime:

```text
script source ──> QuickJsScriptRunner (setup script)
                     |  injects
                     |-- __tyny_modules: preloaded CommonJS-wrapped bundles
                     |-- require(name): registry lookup, clear error if unknown
                     |  sources embedded at compile time
                     v
             assets/script-libs/*.js  (include_str!, zero runtime IO)
```

- Each vendored bundle is wrapped in a CommonJS-shaped IIFE (`module`/`exports` locals), so standard UMD builds select their CJS branch and hand their API to `require`.
- Only **enabled** libraries are preloaded; unknown names raise an actionable error listing installed modules.

## 3. Library Registry

| Module name | Version | Origin | License |
| :--- | :--- | :--- | :--- |
| `lodash` | 4.17.21 | cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.min.js | MIT |
| `dayjs` | 1.x | cdn.jsdelivr.net/npm/dayjs@1/dayjs.min.js | MIT |
| `crypto-js` | 4.x | cdn.jsdelivr.net/npm/crypto-js@4/crypto-js.min.js | MIT |
| `uuid` | 8.3.2 (UMD) | cdn.jsdelivr.net/npm/uuid@8.3.2/dist/umd/uuid.min.js | MIT |

Files live in `src-tauri/assets/script-libs/`; attribution goes to `src-tauri/assets/script-libs/THIRD-PARTY-NOTICES.md`.

## 4. Enable/Disable Settings

- Workspace-scoped JSON file (same base directory convention as the collections repository), written by a small persistence helper: `{ "disabled": ["dayjs"] }`.
- Missing file ⇒ everything enabled (zero-config default).
- The scripting adapter re-reads settings on every execution so UI toggles apply immediately without restarting.

## 5. IPC Surface

| Command | Input | Output |
| :--- | :--- | :--- |
| `list_script_libraries` | — | `Vec<ScriptLibraryInfo>` (name, version, description, enabled) |
| `set_script_library_enabled` | `name: String`, `enabled: bool` | updated `Vec<ScriptLibraryInfo>` |

`ScriptLibraryInfo` derives `TS` (bindings exported through the existing `cargo test export_ts_bindings` pipeline; manual registration step per debt item #4).

## 6. UI

A "Script Libraries" section inside the existing Workspace Settings screen: one row per library (name, version, short description, toggle). Thin React component — mount-time `invoke` + optimistic toggle; no business logic.

## 7. Non-Goals

- No runtime npm download / tarball resolution (deferred; needs new crates + network host approval).
- No per-request library selection; scope is workspace-level.
- No new crates or npm packages (Rule 5 respected; vendored assets are data files).
