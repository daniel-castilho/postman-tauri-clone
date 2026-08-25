# Technical Specification — End-to-End Type Safety & Zero Type-Drift (P1)

**Status:** Draft for Implementation  
**Epic Focus:** Automated TypeScript generation from Rust DTOs across the Tauri v2 IPC boundary  
**Companions:** `zero-type-drift-backlog.md` · `zero-type-drift-implementation-sequence.md` · `ai-software-engineer-prompt-zero-type-drift.md`

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Purpose & Scope

Eliminate manual type duplication and runtime serialization bugs between the Rust core engine and the React 19 frontend by establishing **automated compile-time type generation**.

### In Scope (P1)

- Integration of `ts-rs` crate into `src-tauri/Cargo.toml`.
- Annotation of all Tauri IPC command DTOs with `#[derive(TS)]` and attributes (`#[ts(export, export_to = "../../src/types/generated/")]`).
- Automated TypeScript file generation triggered during `cargo test`.
- Refactoring React 19 UI components and Zustand stores to import generated types from `@/types/generated/`.
- Deprecation and removal of redundant hand-written TypeScript interfaces in `src/types/`.
- CI type-drift protection guard (`git diff --exit-code src/types/generated/`).

### Out of Scope

- Rewriting backend Rust domain models or application use-case logic.
- Replacing Tauri IPC with alternative transport protocols (e.g. gRPC-Web).
- Modifying visual UI layout or Framer Motion animation logic.

---

## 2. Current Vulnerabilities & Type-Drift Risks

| Area                    | Current Implementation                                           | Risk / Problem                                                                          |
| :---------------------- | :--------------------------------------------------------------- | :-------------------------------------------------------------------------------------- |
| **Tauri IPC DTOs**      | Defined as Rust structs in `src-tauri/src/application/commands/` | Changes to field names or optional fields require manual TS updates.                    |
| **Frontend Interfaces** | Hand-written TypeScript interfaces in `src/types/`               | High risk of type drift; field renames in Rust cause runtime `undefined` bugs in React. |
| **Enums & Unions**      | Enums like `HttpMethod` or `ScriptStatus` mirrored manually      | Value mismatches (e.g. `"GET"` vs `"Get"`) fail silently during IPC JSON parsing.       |
| **CI Verification**     | No automated check verifying Rust ↔ TS synchronization           | Out-of-sync types can be merged into `main` unnoticed.                                  |

---

## 3. Target Architecture & Tooling

```
  ┌─────────────────────────────────────────────────────────┐
  │                 RUST BACKEND (src-tauri)                │
  │                                                         │
  │   #[derive(Serialize, Deserialize, TS)]                 │
  │   #[ts(export, export_to = "../../src/types/generated/")]│
  │   pub struct HttpRequestDto { ... }                     │
  └────────────────────────────┬────────────────────────────┘
                               │
                       cargo test / build
                               │ (Automated Export)
                               ▼
  ┌─────────────────────────────────────────────────────────┐
  │              GENERATED TYPESCRIPT DEFINITIONS           │
  │            (src/types/generated/HttpRequestDto.ts)      │
  └────────────────────────────┬────────────────────────────┘
                               │
                               │ (Strict Import)
                               ▼
  ┌─────────────────────────────────────────────────────────┐
  │              REACT 19 FRONTEND (src/App.tsx)            │
  │   import type { HttpRequestDto }                        │
  │     from "@/types/generated/HttpRequestDto";            │
  └─────────────────────────────────────────────────────────┘
```

---

## 4. Required Technical Changes

### 4.1 Dependency Addition (`src-tauri/Cargo.toml`)

Add `ts-rs` to dev-dependencies and dependencies with Tauri compatibility features:

```toml
[dependencies]
ts-rs = { version = "10.1", features = ["serde-json-impl"] }
```

### 4.2 Rust DTO Annotations

Annotate every Tauri IPC input/output DTO across all command modules:

- `HttpRequestDto`, `HttpResponseDto`, `HttpHeadersDto`
- `WorkspaceDto`, `CollectionDto`, `FolderDto`
- `EnvironmentDto`, `EnvironmentVariableDto`
- `VaultItemDto`, `EncryptedVaultDto`
- `SpecLintResultDto`, `SpecDiagnosticDto`

Example annotation pattern:

```rust
use ts_rs::TS;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct HttpRequestDto {
    pub id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<HttpHeaderDto>,
    pub body: Option<String>,
}
```

### 4.3 Automated Export Test Runner

Add a dedicated test module `src-tauri/src/application/commands/export_ts_bindings.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_ts_bindings() {
        HttpRequestDto::export_all().unwrap();
        HttpResponseDto::export_all().unwrap();
        WorkspaceDto::export_all().unwrap();
        CollectionDto::export_all().unwrap();
        EnvironmentDto::export_all().unwrap();
        VaultItemDto::export_all().unwrap();
        SpecLintResultDto::export_all().unwrap();
    }
}
```

### 4.4 Frontend Refactoring (`src/`)

- Update `tsconfig.json` path alias `@/types/generated/*` resolving to `src/types/generated/*`.
- Replace imports in React components and Zustand stores (`src/store/useWorkspaceStore.ts`).
- Remove redundant hand-written `.ts` interface files.

---

## 5. Testing & Validation Requirements

1. **Rust Binding Test:** `cargo test export_ts_bindings` must generate all `.ts` files inside `src/types/generated/`.
2. **Frontend Type Check:** `npm run build` must compile without any TypeScript errors (`tsc --noEmit`).
3. **IPC Functional Verification:** Tauri IPC `invoke()` calls must deserialize correctly with zero runtime errors.
4. **CI Drift Guard:** CI pipeline executes `git diff --exit-code src/types/generated/` after `cargo test` to fail builds if generated types were modified in Rust but uncommitted in Git.

---

## 6. Definition of Done

- [ ] All IPC DTOs annotated with `#[derive(TS)]` and export paths.
- [ ] `src/types/generated/` contains fresh TypeScript files generated by Rust.
- [ ] React components and Zustand stores consume generated types exclusively.
- [ ] `cargo test` and `npm run build` pass 100% green.
- [ ] CI drift guard documented and verified.
