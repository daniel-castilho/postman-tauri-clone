# Testing Playbook — Tyny Pulse

Role: Define how to design, run, diagnose, and maintain tests for **Tyny Pulse** across the Rust core, QuickJS sandbox, AES-256 vault, and React 19 frontend.

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Testing Principles

1. Test observable business rules and contracts, not internal implementation details.
2. Maintain the fastest possible feedback loop at the lowest appropriate layer (`cargo test`).
3. Domain business invariants must be tested in pure Rust without infrastructure dependencies or mocks.
4. Outbound ports (`HttpClientPort`, `VaultPort`, `ScriptEnginePort`) should be mocked during use-case service tests.
5. Every rejection path (invalid URL, network timeout, corrupted vault ciphertext, invalid OpenAPI spec) is as valuable as the happy path.
6. Tests must be deterministic, isolated, and repeatable in local environments and CI workers.
7. Never delete or weaken a valid test merely to pass a build.

---

## 2. Test Taxonomy

| Level                      | Location                        | Runtime                             | Purpose                                                                                                    |
| :------------------------- | :------------------------------ | :---------------------------------- | :--------------------------------------------------------------------------------------------------------- |
| **Domain Unit**            | `src-tauri/src/domain/`         | Pure Rust (`cargo test`)            | Entity invariants, URL validation, Header normalization, DomainError enums. Zero mocks.                    |
| **Application Unit**       | `src-tauri/src/application/`    | Rust + Mock Ports (`cargo test`)    | Use-case orchestration (`ExecuteRequestService`, `LintSpecService`), variable resolution logic.            |
| **Infrastructure Adapter** | `src-tauri/src/infrastructure/` | Rust Integration (`cargo test`)     | `ReqwestAdapter` HTTP calls, `QuickJSAdapter` script evaluations, `AesVaultAdapter` encryption/decryption. |
| **Frontend Typecheck**     | `src/`                          | TypeScript + Vite (`npm run build`) | React 19 component typing, Zustand state updates, IPC DTO contract verification.                           |

---

## 3. Commands & Execution

### 3.1 Fast Local Feedback Loop (Rust Unit Tests)

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Runs pure domain unit tests and application use-case tests in milliseconds.

### 3.2 Linter & Code Quality Gate

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### 3.3 Frontend Build & Type Validation

```bash
npm run build
```

### 3.4 Headless CLI Runner (`tyny-cli`)

```bash
cargo build --manifest-path src-tauri/Cargo.toml --bin tyny-cli
src-tauri/target/debug/tyny-cli run src-tauri/tests/fixtures/sample_collection.json --report target/report.junit
```

Executes a collection without the Tauri GUI over the same application layer used by the desktop app. Exit code contract: `0` all assertions passed, `1` assertion failures, `2` usage/input error, `3` domain error. Reports: JSON envelope (schema version, request/assertion summary, duration) and JUnit XML (`testsuites`/`testsuite`/`testcase`/`failure`). The committed fixture expects the local mock server from `src-tauri/tests/cli_headless.rs`; integration tests cover all three exit paths automatically via `cargo test`.

### 3.5 Git Adapter Integration Tests

```bash
cargo test --test git_adapter --manifest-path src-tauri/Cargo.toml
```

Drives `GitProcessAdapter` (system git subprocess) against temporary repositories: status/stage/commit lifecycle, branch creation and checkout, unified diff parsing, non-repository detection and the automatic `.gitignore` security exclusions that keep `*.vault.enc` / `.vault.key` / `.env.local` out of tracked changes. Tests skip gracefully when git is unavailable on the host.

### 3.6 TypeScript Binding Export & Type-Drift Guard

```bash
cargo test export_ts_bindings --manifest-path src-tauri/Cargo.toml
```

Regenerates every IPC contract binding into `src/types/generated/` from the Rust domain models (single source of truth). The generated files are committed; never edit them by hand.

CI (`.github/workflows/ci.yml`) re-runs this export and fails when `git status --porcelain src/types/generated` is non-empty, meaning a Rust DTO changed without its TypeScript bindings being regenerated. Fix locally with:

```bash
cd src-tauri && cargo test export_ts_bindings
```

---

## 4. Mandatory Patterns & Rules

| Area                      | Rule                                                                                                                       |
| :------------------------ | :------------------------------------------------------------------------------------------------------------------------- |
| **Domain Tests**          | Instantiate pure Rust structs directly. No `reqwest`, `tauri`, or file system mocks.                                       |
| **Application Tests**     | Mock outbound ports (`HttpClientPort`, `VaultPort`, `ScriptEnginePort`). Verify use-case orchestration and error handling. |
| **Secrets & Security**    | Never assert or log plaintext passwords, API keys, or decrypted vault keys inside test outputs.                            |
| **Boundary Verification** | Enforce Rule 1 boundary check. Domain and application ports must never import infrastructure adapters:                     |

```bash
grep -rEn "use crate::infrastructure" src-tauri/src/domain src-tauri/src/application/ports
```

_Expected result: 0 matches._

---

## 5. Regression Checklist

| Area                  | Must Verify                                                                                                  |
| :-------------------- | :----------------------------------------------------------------------------------------------------------- |
| **Request Execution** | HTTP methods (GET, POST, PUT, DELETE, PATCH), header injection, query param encoding, body payload handling. |
| **Scripting Sandbox** | `pm.test()`, `pm.environment.set()`, `pm.response.json()` evaluation in QuickJS sandbox.                     |
| **Vault Encryption**  | AES-256-GCM encryption at rest, key derivation with PBKDF2/Argon2, invalid passphrase rejection.             |
| **SpecHub Linter**    | Real-time diagnostic error reporting for invalid OpenAPI 3.0 / 3.1 specifications.                           |
| **Local Workspace**   | JSON serialization determinism and 2-space formatting for clean Git diffs.                                   |

---

## 6. Definition of Done for Testing

- [ ] New domain logic has a colocated Rust unit test function in `src-tauri/src/domain/`.
- [ ] At least one happy path and one rejection/failure path are tested.
- [ ] `cargo test` passes cleanly without warnings or failures.
- [ ] `cargo clippy -- -D warnings` reports no lint warnings.
- [ ] `npm run build` completes without TypeScript or Vite bundling errors.
- [ ] `cargo test export_ts_bindings` leaves `git status --porcelain src/types/generated` empty (no type drift).
- [ ] Architecture boundary grep returns 0 matches.
