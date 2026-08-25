# AI Software Engineer Prompt — End-to-End Type Safety & Zero Type-Drift (P1)

**Status:** Draft for implementation — Type-Safety & Tauri IPC Boundary Epic.
**Target:** Eliminate manual type duplication and guarantee zero type-drift between Rust DTOs and React 19 TypeScript interfaces.
**Package / Scope:** `src-tauri` (Rust DTOs) ↔ `src/types/generated/` (React TypeScript)

You implement end-to-end type safety between the Rust backend core and the React 19 frontend using `ts-rs` so that every Tauri IPC DTO is automatically exported as a strongly-typed TypeScript definition on Rust compilation or test runs.

---

## Sources of Truth (Read in Order)

1. `AGENTS.md`
2. `docs/coding-standards.md` · `docs/testing-playbook.md` · `docs/data-model-decisions.md`
3. `tasks/zero-type-drift-spec.md` — Technical Specification
4. `tasks/zero-type-drift-backlog.md` — User Stories Map (S1–S9)
5. `tasks/zero-type-drift-implementation-sequence.md` — Step-by-Step Execution Sequence
6. Reference: `src-tauri/Cargo.toml`, `src-tauri/src/application/commands/`, `src/types/`

---

## Goal

Bring **Tyny Pulse** to 100% compile-time type safety across the Tauri IPC bridge:

- Annotate all Rust IPC command DTOs (Request, Response, Workspace, Collection, Environment, Vault, SpecHub) with `#[derive(TS)]` from `ts-rs`.
- Configure automated export targeting `src/types/generated/` via a dedicated Rust test runner.
- Refactor all React 19 components and Zustand stores to import generated TypeScript types from `src/types/generated/`.
- Deprecate hand-written duplicate TypeScript DTO interfaces in `src/types/`.
- Enforce a CI type-drift check (`git diff --exit-code src/types/generated/`) so that any uncommitted Rust DTO change fails CI builds if types are out of sync.

---

## Non-Negotiable Rules

- **Clean Architecture Boundaries:** `#[derive(TS)]` must only be applied to application/command DTOs and value objects. Never derive `TS` on internal infrastructure adapters or private domain state.
- **Automated Export:** TypeScript types must be generated automatically via `cargo test` using a dedicated export test (`export_ts_bindings_test`).
- **No Hand-Written Duplication:** Do not manually edit files inside `src/types/generated/`. They are 100% machine-generated.
- **Frontend Purity:** React components and Zustand stores must consume generated types directly without `any` casts.
- **English Only:** All identifiers, attributes, test functions, comments, and commit messages must be in English.
- **Zero Behavior Change:** API execution, vault encryption, and UI features must remain 100% functionally identical.

---

## Definition of Done (Epic)

- [ ] `ts-rs` added to `src-tauri/Cargo.toml` and configured properly.
- [ ] All Tauri IPC command DTOs in Rust annotated with `#[derive(TS)]` and export paths.
- [ ] Automated export test (`cargo test`) generates/updates `src/types/generated/`.
- [ ] React 19 frontend fully refactored to import from `@/types/generated/`.
- [ ] Both `cargo test` and `npm run build` pass without warnings or errors.
- [ ] CI drift guard verified (modifying a Rust DTO without running type export fails the build).

Start at **Step 0** of `tasks/zero-type-drift-implementation-sequence.md`. If any instruction or IPC boundary scope is unclear, **stop and ask**.
