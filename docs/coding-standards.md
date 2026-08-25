# Coding Standards — Rust / Tauri / React 19 (Tyny Pulse)

Practical reference for solo and AI-assisted development of **Tyny Pulse**. Goal: **consistency over time, not ceremony**. Living document — edit as the project evolves.

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

| Element                           | Convention                 | Example                                                                    |
| :-------------------------------- | :------------------------- | :------------------------------------------------------------------------- |
| **Rust Modules**                  | `snake_case`               | `execute_request_service.rs`, `variable_resolver_adapter.rs`               |
| **Rust Structs & Enums**          | `PascalCase`               | `HttpRequest`, `HttpResponse`, `HttpMethod`, `DomainError`                 |
| **Domain Models & Value Objects** | `PascalCase`; IDs as `*Id` | `Request`, `Collection`, `Environment`, `RequestId`, `WorkspaceId`         |
| **Outbound Ports (Traits)**       | `*Port`                    | `HttpClientPort`, `ScriptEnginePort`, `VaultPort`, `FileSystemPort`        |
| **Infrastructure Adapters**       | `*Adapter`                 | `ReqwestAdapter`, `QuickJSAdapter`, `AesVaultAdapter`, `GitProcessAdapter` |
| **Use-Case Services**             | `*Service`                 | `ExecuteRequestService`, `RunCollectionService`, `LintSpecService`         |
| **Tauri Commands**                | `snake_case`               | `execute_http_request`, `save_workspace`, `open_vault`                     |
| **React Components**              | `PascalCase`               | `RequestEditor.tsx`, `ResponsePanel.tsx`, `WorkspaceSelector.tsx`          |
| **Zustand Stores**                | `use*Store`                | `useWorkspaceStore`, `useEnvironmentStore`, `useThemeStore`                |
| **Custom React Hooks**            | `use*`                     | `useTauriEvent.ts`, `useRequestRunner.ts`                                  |
| **TypeScript DTOs / Interfaces**  | `PascalCase`               | `HttpRequestDto`, `HttpResponseDto`, `WorkspaceDto`                        |
| **CSS Modules**                   | `camelCase` classes        | `styles.sidebarContainer`, `styles.activeTabBadge`                         |
| **Constants**                     | `UPPER_SNAKE_CASE`         | `DEFAULT_TIMEOUT_MS`, `VAULT_SALT_SIZE`                                    |
| **Test Functions**                | `snake_case`               | `should_execute_get_request_successfully`                                  |

Name for **what it is or does**, not the implementation: `HttpClientPort`, not `ReqwestHttpClient` (HTTP client is swappable behind the port). Use-case names speak **business language** (`ExecuteRequest`, `EncryptSecret`), not HTTP verbs or Tauri IPC details.

> **Generated Types Rule:** Never hand-write TypeScript DTOs or interfaces that already exist in `src/types/generated/`. Always import directly from `@/types/generated/` or the `src/types/ipc.ts` barrel.

---

## 2. Package & Directory Boundaries (Clean Architecture)

```
tyny-pulse/
├── src/                          # PRESENTATION LAYER (React 19 + TypeScript)
│   ├── components/               # Atomic UI & Layout components
│   ├── store/                    # Curried Zustand stores (`create<State>()()`)
│   ├── types/                    # Generated TypeScript DTOs (`src/types/generated/`)
│   ├── App.tsx                   # App container & router layout
│   └── main.tsx                  # React entry point
│
└── src-tauri/                    # BACKEND CORE (Rust 2021)
    ├── Cargo.toml                # Rust crate manifest
    ├── tauri.conf.json           # Tauri v2 desktop manifest (ca.tyny.pulse)
    └── src/
        ├── domain/               # Pure Rust entities & port traits (Zero external deps)
        ├── application/          # Use cases, commands & port traits
        ├── infrastructure/       # Concrete adapters (Reqwest, QuickJS, Vault, Git, FS)
        └── main.rs               # Desktop bootstrap & IPC registration
```

### Framework Boundary Rules (Enforced by `AGENTS.md` Rule 1)

| Layer                 | Allowed / Prohibited Imports                                                                                                                                                                                     |
| :-------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`domain/`**         | **Pure Rust Only** — No `reqwest`, no `tauri`, no `rquickjs`, no `serde_json` IO, no file system calls. Entities, value objects, and domain error enums are written in pure Rust.                                |
| **`application/`**    | **Pure Business Logic** — Depends only on `domain` models and `ports/` traits. Imports `serde` for DTO serialization. No `infrastructure` adapters, no `tauri` UI bindings, no raw file system IO.               |
| **`infrastructure/`** | **Full Technology Adapters Allowed** — `reqwest`, `rquickjs`, `aes-gcm`, `tokio`, `std::fs`, `serde_json`. Implements traits defined in `application/ports/` or `domain/`.                                       |
| **`src/` (React)**    | **Presentation Only** — React 19 components, Zustand state, CSS Modules, Framer Motion. Uses Tauri IPC exclusively via generated bindings in `@/types/generated/`. **No business or encryption logic in React.** |

---

## 3. Practical "Do vs Don't" Code Examples

### 3.1 Domain & Port Trait Abstraction

❌ **DON'T (Infrastructure Coupling in Application Service):**

```rust
// BAD: Directly coupling application service to Reqwest HTTP client
use reqwest::Client;

pub struct ExecuteRequestService {
    client: Client, // Direct infrastructure dependency!
}
```

✅ **DO (Clean Architecture Port Injection):**

```rust
// GOOD: Depending on pure application port trait
use crate::application::ports::http_client::HttpClientPort;
use std::sync::Arc;

pub struct ExecuteRequestService {
    http_client: Arc<dyn HttpClientPort>, // Abstract trait port
}
```

---

### 3.2 Protocol Handling (Strategy Pattern vs Monolithic Branching)

❌ **DON'T (Monolithic Branching on Protocol Strings):**

```rust
// BAD: Giant match statement forcing all protocol logic into one function
pub async fn send_payload(protocol: &str, body: &str) -> Result<Response, Error> {
    match protocol {
        "HTTP" => { /* 100 lines of HTTP code */ },
        "WEBSOCKET" => { /* 100 lines of WS code */ },
        "GRPC" => { /* 100 lines of gRPC code */ },
        _ => panic!("Unknown protocol"),
    }
}
```

✅ **DO (Protocol Strategy Trait Dispatch):**

```rust
// GOOD: Polymorphic trait dispatch per protocol
#[async_trait]
pub trait ProtocolStrategy: Send + Sync {
    async fn execute(&self, request: &Request) -> Result<Response, DomainError>;
}

pub struct HttpProtocolStrategy { /* reqwest adapter */ }
pub struct WebSocketProtocolStrategy { /* tungstenite adapter */ }
```

---

### 3.3 Secrets & Token Handling

❌ **DON'T (Logging Raw Secrets or Exposing Unencrypted Plaintext over IPC):**

```rust
// BAD: Printing raw API keys or passwords to logs or IPC stdout
tracing::info!("Authenticating with secret key: {}", api_key);
println!("Decrypted vault password: {}", master_pass);
```

✅ **DO (Zeroizing Memory Wrappers & Structured Log Masking):**

```rust
// GOOD: Secrets remain encrypted at rest and masked in logs
use zeroize::ZeroizeOnDrop;

#[derive(ZeroizeOnDrop)]
pub struct VaultSecret {
    inner: String,
}

// Log contextual metadata only — NEVER the secret payload!
tracing::info!(request_id = %id, user_id = %user, "Authentication request processed");
```

---

### 3.4 Rust Error Handling (`thiserror` vs `unwrap`)

❌ **DON'T (Unsafe Panics in Production Code):**

```rust
// BAD: Panicking or returning unhandled infrastructure errors to IPC
pub fn parse_workspace(json_str: &str) -> Workspace {
    let workspace: Workspace = serde_json::from_str(json_str).unwrap(); // DO NOT UNWRAP IN PROD!
    workspace
}
```

✅ **DO (Structured Domain Errors):**

```rust
// GOOD: Explicit error mapping via DomainError enum
use crate::domain::errors::DomainError;

pub fn parse_workspace(json_str: &str) -> Result<Workspace, DomainError> {
    serde_json::from_str(json_str)
        .map_err(|e| DomainError::InvalidWorkspaceFormat(e.to_string()))
}
```

---

### 3.5 Concurrency & Lock-Free Channels (`mpsc` vs `Mutex`)

❌ **DON'T (Mutex Contention across Tokio Worker Threads):**

```rust
// BAD: Wrapping shared state in Mutex across 100 VUs in load test loop
let results = Arc::new(Mutex::new(Vec::new()));
// Workers fight for Mutex lock on every single HTTP request!
```

✅ **DO (Lock-Free Single Writer via `mpsc` Channel):**

```rust
// GOOD: Workers send minimal Sample structs over Tokio mpsc channel
let (tx, mut rx) = tokio::sync::mpsc::channel::<Sample>(10000);

// Single aggregator task computes RPS and p50/p95/p99 percentiles without lock contention
tokio::spawn(async move {
    while let Some(sample) = rx.recv().await {
        metrics_aggregator.record(sample);
    }
});
```

---

### 3.6 Zustand Store Action Design

❌ **DON'T (Embedding Heavy Business / Transformation Logic in React Store):**

```ts
// BAD: Running complex parsing, variable resolution, and regex inside Zustand action
const useWorkspaceStore = create((set) => ({
  executeRequest: async (req) => {
    // 80 lines of JS regex substitution, auth encoding, and script evaluation in UI store!
  },
}));
```

✅ **DO (Lean Store Actions Delegating to Rust Core via Typed IPC):**

```ts
// GOOD: Store action strictly updates state and delegates execution to Rust core
export const useWorkspaceStore = create<WorkspaceState>()((set) => ({
  executeRequest: async (requestDto: HttpRequestDto) => {
    set({ isExecuting: true });
    const result = await invoke<SendRequestOutput>('send_request', { request: requestDto });
    set({ activeResponse: result.response, isExecuting: false });
  },
}));
```

---

### 3.7 Styling with CSS Modules & Custom Glassmorphism

❌ **DON'T (Inline Style Objects & Hardcoded Magic Values):**

```tsx
// BAD: Messy inline styling bypassing design tokens
<div style={{ background: 'rgba(255,255,255,0.1)', padding: '12px 20px', borderRadius: '8px' }}>
  <span>Header</span>
</div>
```

✅ **DO (Scoped CSS Modules with Scoped Tokens):**

```tsx
// GOOD: Importing scoped CSS module
import styles from './WorkspaceSelector.module.css';

export function WorkspaceSelector() {
  return (
    <div className={styles.cardContainer}>
      <span className={styles.cardHeader}>Header</span>
    </div>
  );
}
```

---

## 4. Deep-Dive Rust & Tokio Directives

1. **Async Task Spawning:** Always spawn background work using `tokio::spawn` or `tokio::task::spawn_blocking` (for heavy CPU/file I/O). Never block the main thread.
2. **Cancellation Discipline:** Long-running tasks (load tests, collection runs) must listen to a `tokio_util::sync::CancellationToken` to halt worker loops within < 100ms when user clicks Abort.
3. **Lifetimes & Thread Safety:** Structs shared across Tokio tasks must satisfy `Send + Sync + 'static`. Wrap shared port dependencies in `Arc<dyn PortTrait>`.

---

## 5. Deep-Dive React 19 & TypeScript Directives

1. **Curried Zustand Store Pattern:** All Zustand stores in `src/store/` must use the curried type syntax to prevent implicit `any` state inferencing:
   ```ts
   export const useWorkspaceStore = create<WorkspaceState>()((set, get) => ({
     // Store state & actions
   }));
   ```
2. **Event Listener Cleanup:** Custom hooks subscribing to Tauri events (`app_handle.listen()`) must unlisten in their cleanup return function to prevent event memory leaks.

---

## 6. Secrets & Security Discipline

- **Vault Encrypted Isolation:** Plaintext secrets (API keys, passwords, bearer tokens) must never leave the `VaultPort` / AES adapter in unencrypted storage.
- **IPC Payload Safety:** Never serialize decrypted vault master keys or unencrypted secret stores into TypeScript DTOs.
- **Log Masking:** Never print, log, or include raw secret values in error messages or stdout tracing.
- **Git Protection:** Ensure workspace `.gitignore` excludes `.vault.enc`, `.vault.key`, `.env.local`, and local overrides.

---

## 7. Testing Strategy & Coverage Expectations

| Kind                       | Tooling                  | Scope                                      | Coverage Floor | Mocks Allowed?                                   |
| :------------------------- | :----------------------- | :----------------------------------------- | :------------- | :----------------------------------------------- |
| **Domain & Application**   | `cargo test`             | Pure Rust models, Value Objects, Use Cases | **80%** lines  | **Mock ONLY Ports.** Never mock domain entities. |
| **Infrastructure Adapter** | `cargo test`             | Real file system, QuickJS, Git subprocess  | **70%** lines  | Real temp directories (`tempfile` crate).        |
| **Frontend Stores & UI**   | Vitest / `npm run build` | Zustand stores, DTO mappings, Components   | **75%** lines  | Mock Tauri IPC `invoke()` bindings.              |

### Coverage Verification Command

```bash
cargo llvm-cov --fail-under-lines 80 --quiet
```

Prefer testing **observable behavior and invariants** over internal implementation details.

---

## 8. Formatting, Tooling & Pre-Commit Quality Gate

- **Rust:** `rustfmt` (edition 2021 defaults) + `clippy --all-targets -- -D warnings`
- **TypeScript:** Prettier + ESLint 9 Flat Config (`eslint.config.mjs`)
- **Indentation:** 4 spaces (Rust), 2 spaces (TypeScript/CSS)
- **Line Length Soft Limit:** 100 characters (hard limit 120)

Every commit must satisfy the full local quality gate:

```bash
# 1. Rust Formatting & Linter (Strict Warnings Gate)
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

# 2. Rust Binding Export Test
cargo test --manifest-path src-tauri/Cargo.toml export_ts_bindings

# 3. Code Coverage Gate
cargo llvm-cov --manifest-path src-tauri/Cargo.toml --fail-under-lines 80 --quiet

# 4. Frontend Formatting, ESLint Flat Config & Type Check
npm run format:check
npm run lint
npm run build
```

---

## Quick Pre-Commit Checklist

- [ ] `domain/` and `application/ports/` are free of `infrastructure` imports (Rule 1 grep returns 0).
- [ ] No `unwrap()` or `expect()` in non-test Rust files.
- [ ] No raw secrets logged or exposed over Tauri IPC.
- [ ] React components consume generated types from `@/types/generated/`.
- [ ] `cargo clippy --all-targets -- -D warnings`, `npm run lint` and `npm run build` execute without errors or warnings.
- [ ] Documentation updated if a milestone feature was added or modified.

---

> _"When in doubt, prefer the simpler solution that still respects the layer boundaries. Cleverness is debt."_
