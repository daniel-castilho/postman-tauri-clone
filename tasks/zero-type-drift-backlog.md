# End-to-End Type Safety & Zero Type-Drift — Backlog (P1)

**Companions:** `zero-type-drift-spec.md` · `zero-type-drift-implementation-sequence.md` · `ai-software-engineer-prompt-zero-type-drift.md`  
**Epic Goal:** Eliminate manual type duplication and guarantee zero type-drift between Rust DTOs and React 19 TypeScript interfaces.

**MVP Scope:** Stories S1–S9

---

## Story Map

```text
AUDIT & SETUP
S1 Audit all Tauri IPC commands and DTOs across Rust and React
S2 Add `ts-rs` dependency to Cargo.toml and configure export directories

RUST DTO ANNOTATIONS
S3 Annotate HTTP Request & Response DTOs with #[derive(TS)]
S4 Annotate Workspace, Collection, & Folder DTOs with #[derive(TS)]
S5 Annotate Environment, Secret Vault, & SpecHub DTOs with #[derive(TS)]

AUTOMATION & FRONTEND MIGRATION
S6 Create automated export test runner (`export_ts_bindings_test`)
S7 Refactor React components & Zustand stores to consume `@/types/generated/`
S8 Remove redundant hand-written TypeScript interfaces

CI GUARD & VERIFICATION
S9 Implement CI type-drift verification step & validate regression suite
```

---

## Stories Breakdown

| ID     | Story Title                                                                | Priority | Target Modules / Components                                        | Notes                                                                       |
| :----- | :------------------------------------------------------------------------- | :------- | :----------------------------------------------------------------- | :-------------------------------------------------------------------------- |
| **S1** | Audit all Tauri IPC commands and DTOs across Rust and React                | Must     | `src-tauri/src/application/commands/`, `src/types/`                | Create inventory of all IPC structs and hand-written TS types               |
| **S2** | Add `ts-rs` dependency to Cargo.toml and configure export paths            | Must     | `src-tauri/Cargo.toml`                                             | Feature configuration (`serde-json-impl`)                                   |
| **S3** | Annotate HTTP Request & Response DTOs with `#[derive(TS)]`                 | Must     | `src-tauri/src/application/commands/http_dtos.rs`                  | Includes `HttpMethod`, `HttpHeaderDto`, `HttpRequestDto`, `HttpResponseDto` |
| **S4** | Annotate Workspace, Collection, & Folder DTOs with `#[derive(TS)]`         | Must     | `src-tauri/src/application/commands/workspace_dtos.rs`             | Includes `WorkspaceDto`, `CollectionDto`, `FolderDto`                       |
| **S5** | Annotate Environment, Secret Vault, & SpecHub DTOs with `#[derive(TS)]`    | Must     | `src-tauri/src/application/commands/vault_dtos.rs`, `spec_dtos.rs` | Includes `EnvironmentDto`, `VaultItemDto`, `SpecLintResultDto`              |
| **S6** | Create automated export test runner (`export_ts_bindings_test`)            | Must     | `src-tauri/src/application/commands/export_ts_bindings.rs`         | Executes `export_all()` on all DTOs targeting `src/types/generated/`        |
| **S7** | Refactor React components & Zustand stores to consume `@/types/generated/` | Must     | `src/App.tsx`, `src/store/`, `src/components/`                     | Replace legacy imports with generated TypeScript imports                    |
| **S8** | Remove redundant hand-written TypeScript interfaces                        | Must     | `src/types/`                                                       | Clean up duplicate `.ts` interface declarations                             |
| **S9** | Implement CI type-drift verification step & validate regression suite      | Must     | `.github/workflows/ci.yml`, `docs/testing-playbook.md`             | Add `git diff --exit-code src/types/generated/` gate                        |

---

## Definition of Done (Epic)

- [ ] S1–S9 completed and verified.
- [ ] Automated export test outputs fresh `.ts` files inside `src/types/generated/`.
- [ ] Hand-written duplicate TypeScript interfaces removed.
- [ ] `cargo test` passes green.
- [ ] `npm run build` completes with zero TypeScript compilation errors.
- [ ] Modifying a Rust DTO without running type export fails the CI build guard.
