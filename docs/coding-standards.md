# Coding Standards — Rust / Tauri / React 19 (Tyny Pulse)

Practical reference for solo and AI-assisted development of **Tyny Pulse**. Goal: **consistency over time**, not ceremony. Living document — edit as the project evolves.

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

**Relationship to other docs:**

| Doc               | Wins when                                                 |
| :---------------- | :-------------------------------------------------------- |
| `AGENTS.md`       | Project conventions, release flow, hard agent rules       |
| **This file**     | Day-to-day coding detail that does not fit in `AGENTS.md` |
| `docs/lessons.md` | Durable rules learned the hard way                        |

Where this file conflicts with `AGENTS.md`, **`AGENTS.md` wins**.

---

## 1. Naming Conventions

| Element                           | Convention                 | Example                                                                        |
| :-------------------------------- | :------------------------- | :----------------------------------------------------------------------------- |
| **Rust Modules**                  | `snake_case`               | `execute_request_service.rs`, `variable_resolver_adapter.rs`                   |
| **Rust Structs & Enums**          | `PascalCase`               | `HttpRequest`, `HttpResponse`, `HttpMethod`, `DomainError`                     |
| **Domain Models & Value Objects** | `PascalCase`; IDs as `*Id` | `Request`, `Collection`, `Environment`, `RequestId`, `WorkspaceId`             |
| **Outbound Ports (Traits)**       | `*Port`                    | `HttpClientPort`, `ScriptEnginePort`, `VaultPort`, `FileSystemPort`            |
| **Infrastructure Adapters**       | `*Adapter`                 | `ReqwestAdapter`, `QuickJSAdapter`, `AesVaultAdapter`, `JsonFileSystemAdapter` |
| **Use-Case Services**             | `*Service`                 | `ExecuteRequestService`, `RunCollectionService`, `LintSpecService`             |
| **Tauri Commands**                | `snake_case`               | `execute_http_request`, `save_workspace`, `open_vault`                         |
| **React Components**              | `PascalCase`               | `RequestEditor.tsx`, `ResponsePanel.tsx`, `WorkspaceSelector.tsx`              |
| **Zustand Stores**                | `use*Store`                | `useWorkspaceStore`, `useEnvironmentStore`, `useThemeStore`                    |
| **TypeScript DTOs / Interfaces**  | `PascalCase`               | `HttpRequestDto`, `HttpResponseDto`, `WorkspaceDto`                            |
| **Constants**                     | `UPPER_SNAKE_CASE`         | `DEFAULT_TIMEOUT_MS`, `VAULT_SALT_SIZE`                                        |
| **Test Functions**                | `snake_case`               | `should_execute_get_request_successfully`                                      |

Name for **what it is or does**, not the implementation: `HttpClientPort`, not `ReqwestHttpClient` (HTTP client is swappable behind the port). Use-case names speak **business language** (`ExecuteRequest`, `EncryptSecret`), not HTTP verbs or Tauri IPC details.

---

## 2. Package / Folder Structure (Clean Architecture)

```
tyny-pulse/
├── src/                          # PRESENTATION LAYER (React 19 + TypeScript)
│   ├── components/               # Atomic UI components
│   ├── store/                    # Zustand global UI state
│   ├── types/                    # TypeScript interfaces & DTOs
│   ├── App.tsx                   # App container & router layout
│   └── main.tsx                  # React entry point
│
└── src-tauri/                    # BACKEND CORE (Rust 2021)
    ├── Cargo.toml                # Rust crate manifest
    ├── tauri.conf.json           # Tauri v2 desktop manifest (ca.tyny.pulse)
    └── src/
        ├── domain/               # Pure Rust entities & port traits (Zero external deps)
        ├── application/          # Use cases, commands & port traits
        ├── infrastructure/       # Concrete adapters (Reqwest, QuickJS, AES Vault, FileSystem)
        └── main.rs               # Desktop bootstrap & IPC registration
```

### Framework Boundary (Enforced by `AGENTS.md` Rule 1)

| Layer                 | Allowed / Prohibited Imports                                                                                                                                                                                           |
| :-------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`domain/`**         | **Pure Rust Only** — No `reqwest`, no `tauri`, no `rquickjs`, no `serde_json` IO, no file system calls. Entities, value objects, and domain error enums are written in pure Rust.                                      |
| **`application/`**    | **Pure Business Logic** — Depends only on `domain` models and `ports/` traits. Imports `serde` for DTO serialization. No `infrastructure` adapters, no `tauri` UI bindings, no raw file system IO.                     |
| **`infrastructure/`** | **Full Technology Adapters Allowed** — `reqwest`, `rquickjs`, `aes-gcm`, `tokio`, `std::fs`, `serde_json`. Implements traits defined in `application/ports/` or `domain/`.                                             |
| **`src/` (React)**    | **Presentation Only** — React 19 components, Zustand state, CSS Modules (custom glassmorphism styles), Framer Motion. Uses Tauri `invoke()` exclusively via typed hooks. **No business or encryption logic in React.** |

Tauri IPC command handlers in `application/commands/` are **thin**: they deserialize IPC JSON payloads, delegate execution to an application service (`*Service`), and map the result back to an IPC DTO response.

---

## 3. Clean Architecture & SOLID Principles

- **Dependency Inversion (DIP):** Depend on **ports (traits), never concrete adapters**. Application services consume `HttpClientPort` or `VaultPort`. The concrete `ReqwestAdapter` or `AesVaultAdapter` implements the trait.
- **Interface Segregation (ISP):** Prefer narrow, focused traits over monolithic ones. Split traits like `VaultReaderPort` and `VaultWriterPort` when a use case only requires read access.
- **Open/Closed Principle (OCP):** Use trait dispatch or Strategy pattern for behavior that varies by protocol (REST, GraphQL, WebSocket, gRPC). Avoid deep `match` or `if/else` branching across protocols.
- **Single Responsibility (SRP):** One responsibility per struct/service. Keep methods small and extract helper functions. Name methods as active verbs (`execute_request`, `resolve_variables`).
- **Rich Domain Model:** Enforce invariants in the domain aggregate root (`Request` validates header formats and URL syntax before execution). Avoid anemic models that act as data containers without validation.
- **Exceptions & Errors over Status Flags:** Domain failures return `Result<T, DomainError>`. Never return magic error strings or sentinel `null`/`-1` values.

---

## 4. Design Patterns (House Style)

House patterns are established across the codebase — **reuse them instead of inventing variants**.

| Pattern                        | Where it lives                        | Use it when                                                                    | Avoid / Instead                                                        |
| :----------------------------- | :------------------------------------ | :----------------------------------------------------------------------------- | :--------------------------------------------------------------------- |
| **Adapter (Ports & Adapters)** | `src-tauri/src/infrastructure/`       | Crossing technical boundaries (HTTP, QuickJS, AES-256, FileSystem)             | Calling `reqwest` or `std::fs` directly inside business services       |
| **Strategy**                   | `src-tauri/src/infrastructure/`       | Executing requests across different protocols (REST vs GraphQL vs WebSocket)   | Giant `match` statements in a single execution service                 |
| **Facade (Use-Case Service)**  | `src-tauri/src/application/services/` | Orchestrating domain entities and ports for a single business use case         | Putting orchestration logic directly inside Tauri IPC command handlers |
| **Builder**                    | Domain value objects & requests       | Creating complex `HttpRequest` objects with optional headers, params, and body | Telescoping functions with 10 arguments                                |
| **DTO Translation**            | `src-tauri/src/application/commands/` | Mapping domain entities to/from IPC-safe TypeScript DTOs                       | Exposing domain entities containing unencrypted secrets over Tauri IPC |

---

## 5. Rust Language Directives

- **Error Handling:** Use `thiserror` for structured `DomainError` enums and `anyhow` for infrastructure context wrapping.
- **No Production `unwrap()` / `expect()`:** Production Rust code must return `Result<T, E>`. `unwrap()` is restricted to `#[cfg(test)]` test suites.
- **Immutability & Ownership:** Borrow by reference (`&T`, `&str`) where possible. Use `Arc<T>` for thread-safe shared references across Tokio async tasks.
- **Null / Optional Discipline:** Use `Option<T>` for optional values. Use `Option::ok_or` to convert missing values to explicit `DomainError` variants.

---

## 6. React 19 & TypeScript Directives

- **Strict TypeScript:** `strict: true` in `tsconfig.json`. Explicitly type all component props, Zustand state interfaces, and API DTOs. Prohibit `any`.
- **Zustand State Management:** Keep UI state (active tab, selected workspace, theme mode) inside Zustand stores (`src/store/`). Component-local state (modal visibility, form input) stays in `useState`.
- **Pure Component Render:** Components must remain pure rendering functions. All IPC calls (`invoke()`) must be encapsulated within custom hooks or store actions.

---

## 7. Formatting & Tooling

- 2-space indent for TypeScript/React, 4-space indent for Rust.
- Run `cargo clippy -- -D warnings` before committing Rust code.
- Run `npm run build` to verify TypeScript types and Vite bundle integrity before committing.

---

## 8. Errors & Logging Discipline

- **Structured Domain Errors:** Map infrastructure failures (reqwest network error, QuickJS runtime exception, AES decryption error) into explicit `DomainError` variants before returning to the frontend.
- **No Secrets in Logs:** Never print raw API keys, passwords, or decrypted secrets in `println!`, `eprintln!`, or `console.log`.
- **Contextual Logs:** Include request IDs, HTTP status codes, and execution duration when logging errors.

---

## Quick Pre-Commit Checklist

- [ ] `domain/` and `application/ports/` are free of `infrastructure` imports (Rule 1 grep returns 0).
- [ ] No `unwrap()` or `expect()` in non-test Rust files.
- [ ] No raw secrets logged or exposed over Tauri IPC.
- [ ] `cargo check` and `npm run build` execute without errors or warnings.
- [ ] Documentation updated if a milestone feature was added or modified.
