# AGENTS.md — Guidelines for AI & Human Contributors

**Tyny Pulse** — an ultra-fast, local-first, cross-platform desktop API client built with **Tauri v2 + Rust (2021) + React 19 (TypeScript)** implementing strict **Clean Architecture** and **SOLID principles**.

- **Official Domain:** [https://tyny.ca](https://tyny.ca)
- **App Identifier:** `ca.tyny.pulse`
- **Target Platforms:** Windows, macOS, Linux (desktop native)

Sources of truth: `README.md`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `docs/progress.md`. Re-read the relevant parts before starting any task.

---

## 🚫 Critical Rules (Never Violate)

1. **Architecture Boundaries (Rust Backend):**
   `domain/` and `application/ports/` must **never import `infrastructure` or framework-specific code** (`tauri`, `reqwest`, `rquickjs`, `serde_json` IO, or file system adapters).
   *Verification command before declaring task done:*
   ```bash
   grep -rEn "use crate::infrastructure" src-tauri/src/domain src-tauri/src/application/ports
   ```
   *Must return 0 matches.*

2. **Frontend UI Boundary (React 19):**
   Business logic lives strictly in the Rust core (`domain` and `application` layers). **React components must remain thin presentation wrappers.** No HTTP requests, scripting evaluations, or secret encryption logic inside React components. React components only dispatch state updates via Zustand or call Tauri IPC `invoke()` handlers.

3. **Zero Raw Secrets in Logs or State:**
   Never log plaintext API keys, JWTs, or passwords to `println!`, `eprintln!`, `console.log`, or unencrypted local storage. Secrets must always be passed through the local AES-256-GCM encrypted vault port.

4. **No Direct Production `unwrap()` / `panic!()` in Rust:**
   Production Rust code in `application/` and `infrastructure/` must return `Result<T, E>` using structured domain error types (`thiserror` / `anyhow`). `unwrap()` and `expect()` are restricted to unit test suites (`#[cfg(test)]`).

5. **No Unapproved Dependencies:**
   Do **not** add new Rust crates (`Cargo.toml`) or npm packages (`package.json`) without explicit human approval.

6. **English Only in Codebase:**
   All identifiers, variable names, comments, commit messages, documentation, and error codes must be written in English.

7. **Doc Sync is Part of "Done":**
   After completing any feature, architectural change, or bug fix, you MUST update:
   - `README.md` (Features / Roadmap if applicable)
   - `docs/progress.md` (Progress status)
   - `AGENTS.md` (Clear or update "Known technical debt" if touched)
   *Work is NOT done while documentation describes a stale state.*

8. **Test Suite Integrity:**
   `cargo check` and `npm run build` must pass without errors or warnings before declaring any turn or commit completed.

---

## 🛠️ Commands Matrix

| Purpose | Command | Location |
| :--- | :--- | :--- |
| **Run Dev Application (Tauri + Vite)** | `npm run tauri dev` | Root |
| **Run Frontend Dev Server Only** | `npm run dev` | Root |
| **Typecheck & Web Build** | `npm run build` | Root |
| **Build Production Native Package** | `npm run tauri build` | Root |
| **Rust Fast Compile Check** | `cargo check` | `src-tauri/` |
| **Run Pure Rust Unit Tests** | `cargo test` | `src-tauri/` |
| **Run Rust Linter (Clippy)** | `cargo clippy -- -D warnings` | `src-tauri/` |
| **Automated Project Renaming** | `./rename_to_tyny_pulse.sh` | Root |

---

## 🏗️ Architecture & Layer Responsibilities

The application follows **Clean Architecture**, split into four layers with a strict dependency rule that always points **inward**:

```
tyny-pulse/
├── src/                          # PRESENTATION LAYER (React 19 + TypeScript)
│   ├── components/               # Atomic UI components (Buttons, Inputs, Tabs)
│   ├── store/                    # Zustand global UI state (Tabs, Active Workspace)
│   ├── types/                    # TypeScript interfaces & DTOs
│   ├── App.tsx                   # App container & layout
│   └── main.tsx                  # React entry point
│
└── src-tauri/                    # BACKEND CORE (Rust 2021)
    ├── Cargo.toml                # Rust crate manifest
    ├── tauri.conf.json           # Tauri v2 desktop manifest (ca.tyny.pulse)
    └── src/
        ├── domain/               # [Clean Arch: Entities] Pure Rust models & port traits. Zero external deps.
        │   ├── models/           # Request, Collection, Environment, SecretVault entities
        │   ├── value_objects/    # HttpMethod, Url, Headers, Status
        │   └── errors/           # DomainError enum
        │
        ├── application/          # [Clean Arch: Use Cases] Application services & port traits.
        │   ├── commands/         # Tauri IPC command handlers (thin adapters)
        │   ├── ports/            # Trait definitions (HttpClientPort, ScriptEnginePort, VaultPort)
        │   └── services/         # ExecuteRequestService, RunCollectionService, SpecLintService
        │
        ├── infrastructure/       # [Clean Arch: Framework Adapters] Concrete implementations.
        │   ├── http/             # Reqwest HTTP adapter
        │   ├── scripting/        # QuickJS JavaScript sandbox adapter (pm.* API)
        │   ├── security/         # AES-256-GCM local vault adapter
        │   ├── persistence/      # Git-friendly local JSON FileSystem adapter
        │   └── docs/             # Markdown / OpenAPI generator adapters
        │
        └── main.rs               # Desktop application bootstrap & IPC registration
```

### Layer Rules
- **Domain (`src-tauri/src/domain`)**: Pure Rust entities, value objects, and domain error definitions. No `reqwest`, no `tauri`, no `serde_json` IO.
- **Application (`src-tauri/src/application`)**: Use-case orchestration (`ExecuteRequestService`, `LintOpenApiSpecService`). Implements business workflows against `ports/` traits.
- **Infrastructure (`src-tauri/src/infrastructure`)**: Concrete adapters (`ReqwestAdapter`, `QuickJSScriptAdapter`, `AesVaultAdapter`, `JsonFileSystemAdapter`).
- **Presentation (`src/` & Tauri Commands)**: React 19 UI + Tauri IPC command wrappers (`#[tauri::command]`). Translates IPC JSON DTOs to/from application use-case calls.

---

## 📐 Conventions & Standards

### Rust Conventions
- **Naming:** 
  - Modules: `snake_case` (e.g. `execute_request_service.rs`)
  - Structs/Enums: `PascalCase` (e.g. `HttpRequest`, `HttpMethod`)
  - Traits (Ports): `*Port` (e.g. `HttpClientPort`, `ScriptEnginePort`, `VaultPort`)
  - Adapters: `*Adapter` (e.g. `ReqwestAdapter`, `QuickJSAdapter`)
- **Error Handling:** Map all infrastructure errors (network, IO, QuickJS runtime) into explicit `DomainError` variants before surfacing them to the application layer or Tauri IPC.
- **DTOs for IPC:** Never return raw domain entities over Tauri IPC if they contain internal secrets. Always map to explicit IPC response DTOs.

### TypeScript / React Conventions
- **Strict Typing:** `strict: true` in `tsconfig.json`. Prohibit `any` — use explicit interfaces or `unknown`.
- **UI State:** Use Zustand (`src/store/`) for global state (active workspace, open tabs, active theme). Keep component-local UI state in `useState`.
- **Icons & Styling:** Lucide React icons, Tailwind CSS / CSS Modules, Framer Motion for smooth animations.

---

## 🧪 Testing Strategy

- **Rust Unit Tests (`src-tauri/src/domain/`, `src-tauri/src/application/`)**:
  Test entities, value objects, and use-case services in isolation using mock ports.
  Run with: `cargo test`
- **Infrastructure Integration Tests (`src-tauri/src/infrastructure/`)**:
  Test concrete adapters (`ReqwestAdapter`, `QuickJSAdapter`, `AesVaultAdapter`) against mock servers or local temporary files.
- **Frontend Type & Build Check**:
  Run `npm run build` to verify TypeScript types and Vite bundle integrity.

---

## 📝 Commit & Git Standards

Follow **Conventional Commits**:
- `feat:` New capability or user-facing feature
- `fix:` Bug fix or error resolution
- `refactor:` Code restructuring without changing behaviour
- `docs:` Documentation updates (`README.md`, `AGENTS.md`, `progress.md`)
- `chore:` Dependency or build script updates

---

## 📑 Known Technical Debt (Traceability Matrix)

Items currently deferred or awaiting optimization. Do **not** silently introduce new debt — flag items here:

1. **QuickJS WASM/C Bindings Compilation:** Ensure cross-compilation target libraries for Windows (`x86_64-pc-windows-msvc`) and macOS (`aarch64-apple-darwin`) are tested in CI release workflow.
2. **OpenAPI 3.1 Spec Schema Expansion:** SpecHub currently lints OpenAPI 3.0 specs natively; 3.1 JSON Schema dialect validation rules to be expanded in Phase 7.
3. **Workspace File Watcher:** FileSystem adapter currently reads JSON files on demand; real-time file watcher (`notify` crate) planned for live multi-window sync.
4. **Manual Binding Registration:** New IPC types must be added to the export list in `src-tauri/src/application/commands/export_ts_bindings.rs`; consider automating via a proc-macro or build script inventory.
5. **UI Strings Still Portuguese:** The Rule 6 English-only sweep covered all code comments; user-facing UI strings in React components and AI prompt strings in `gemini_adapter.rs` remain Portuguese pending a product-copy decision.
6. **Loose `any` Types at Store Edge:** `workspaceStore.ts` still uses `(state: any)` / `(c: any)` casts inside Zustand setters and event listeners; typing these against generated contracts is incremental follow-up work.
7. **CLI stdout on Windows Release Builds:** The binary sets `windows_subsystem = "windows"`, so headless CLI output may not attach to some Windows shells in release builds. Mitigation deferred: a dedicated `tyny-pulse-cli` bin target without the GUI subsystem attribute (see `tasks/headless-cli-spec.md`, section 7).

---

## 🔍 Operational Discipline & Debugging Guidelines

- **Investigate Before Trial-and-Error:** When encountering Rust compiler or Tauri IPC errors, inspect full stack traces and verify trait bounds before making code changes.
- **Isolate Reproductions:** When an IPC command fails, test the underlying Rust use-case via a `#[test]` function first to separate Tauri IPC serialization issues from business logic bugs.
- **Clean Workspace:** Never commit temporary test artifacts, compiled binaries (`src-tauri/target/`), or `.env` secret files.

---

Designed & Maintained with ❤️ for [Tyny.ca](https://tyny.ca).
