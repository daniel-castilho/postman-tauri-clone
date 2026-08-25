# Technical Specification — Git-Native Interface (P3)

**Status:** Draft for Implementation  
**Epic Focus:** In-app visual Git control panel and backend Git repository adapter for cloud-free workspace collaboration  
**Companions:** `git-native-interface-backlog.md` · `git-native-interface-implementation-sequence.md` · `ai-software-engineer-prompt-git-native-interface.md`

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Purpose & Scope

Provide a **cloud-free, zero-lock-in collaboration experience** for **Tyny Pulse** by embedding native Git version control capabilities directly into the desktop application. Developers can track changes, review visual JSON diffs, commit, pull, and push workspace collections and environments using their own Git remotes (GitHub, GitLab, Bitbucket, self-hosted) without relying on proprietary cloud backends.

### In Scope (P3)

- Pure Rust domain port `GitRepositoryPort` in `src-tauri/src/application/ports/git_repository.rs`.
- Concrete Git adapter (`git2` crate or subprocess `GitProcessAdapter`) in `src-tauri/src/infrastructure/git/`.
- Tauri IPC command wrappers in `src-tauri/src/application/commands/git_tasks.rs`.
- Strongly-typed DTOs annotated with `#[derive(TS)]` exported to `src/types/generated/`.
- React 19 visual Git Panel (`src/components/GitPanel/`):
  - Active branch indicator & branch switcher/creation modal.
  - Staged & unstaged file status list with single/bulk stage & unstage buttons.
  - Commit message editor (`summary` + `description`) with `Ctrl+Enter` shortcut.
  - Sync actions: Push to remote, Pull from remote (with rebase/merge strategy choice), Fetch status.
  - Ahead/behind commit counter indicators (`↑2 ↓0`).
  - Interactive visual JSON diff modal comparing HEAD vs Working Directory / Index.
- Automatic workspace `.gitignore` generator preventing accidental commit of local vaults or machine overrides.

### Out of Scope

- Full interactive 3-way conflict resolution editor (conflicts flag files for manual editor resolution or simple ours/theirs choice in P3).
- Built-in SSH key generation / agent manager (uses system SSH agent / HTTPS credential helper).

---

## 2. Architecture & Data Flow

```
  ┌─────────────────────────────────────────────────────────────┐
  │                 REACT 19 FRONTEND UI                        │
  │   src/components/GitPanel/ (Status, Commit, Diff, Branches)  │
  └──────────────────────────────┬──────────────────────────────┘
                                 │
                     Tauri IPC Commands (`invoke`)
                     (Typed via `@/types/generated/`)
                                 │
                                 ▼
  ┌─────────────────────────────────────────────────────────────┐
  │               APPLICATION LAYER (Rust)                      │
  │   src-tauri/src/application/commands/git_tasks.rs         │
  └──────────────────────────────┬──────────────────────────────┘
                                 │
                     Consumes Trait Abstraction
                                 │
                                 ▼
  ┌─────────────────────────────────────────────────────────────┐
  │         GitRepositoryPort Trait (application/ports)          │
  └──────────────────────────────┬──────────────────────────────┘
                                 │
                     Implemented By Concrete Adapter
                                 │
                                 ▼
  ┌─────────────────────────────────────────────────────────────┐
  │         Git2Adapter / Subprocess (infrastructure/git)       │
  │   (Interacts asynchronously with workspace `.git` folder)   │
  └─────────────────────────────────────────────────────────────┘
```

---

## 3. Data Models & IPC DTOs

### 3.1 Domain & DTO Definitions (`src-tauri/src/domain/models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum GitFileStatusType {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Conflicted,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct GitFileChangeDto {
    pub path: String,
    pub status: GitFileStatusType,
    pub is_staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct GitStatusSummaryDto {
    pub is_repository: bool,
    pub current_branch: String,
    pub branches: Vec<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub files: Vec<GitFileChangeDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct GitDiffChunkDto {
    pub old_line_number: Option<u32>,
    pub new_line_number: Option<u32>,
    pub content: String,
    pub change_type: String, // "add", "delete", "context"
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct GitFileDiffDto {
    pub path: String,
    pub chunks: Vec<GitDiffChunkDto>,
}
```

---

## 4. Port Interface Contract (`GitRepositoryPort`)

```rust
#[async_trait]
pub trait GitRepositoryPort: Send + Sync {
    async fn get_status(&self, workspace_path: &str) -> Result<GitStatusSummaryDto, DomainError>;
    async fn stage_path(&self, workspace_path: &str, relative_path: &str) -> Result<(), DomainError>;
    async fn unstage_path(&self, workspace_path: &str, relative_path: &str) -> Result<(), DomainError>;
    async fn stage_all(&self, workspace_path: &str) -> Result<(), DomainError>;
    async fn commit(&self, workspace_path: &str, message: &str) -> Result<String, DomainError>;
    async fn push(&self, workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError>;
    async fn pull(&self, workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError>;
    async fn checkout_branch(&self, workspace_path: &str, branch_name: &str, create_if_missing: bool) -> Result<(), DomainError>;
    async fn get_diff(&self, workspace_path: &str, relative_path: &str, staged: bool) -> Result<GitFileDiffDto, DomainError>;
}
```

---

## 5. Security & Safety Rules

1. **Gitignore Protection:** When a workspace is opened or initialized as a Git repository, Tyny Pulse creates/verifies a `.gitignore` containing:
   ```gitignore
   # Tyny Pulse Security Exclusions
   *.vault.enc
   .vault.key
   .env.local
   script-libraries-local.json
   ```
2. **Non-Blocking Async Threads:** Git fetch/pull/push operations must execute off the main thread using `tokio::task::spawn_blocking` to ensure zero UI freezes during network handshakes.
3. **Deterministic Serialization:** Collection JSON writes maintain 2-space key ordering so Git diffs compare clean property lines instead of reformatted minified blocks.

---

## 6. Testing & Validation Requirements

1. **Adapter Unit Tests:** Test repository initialization, staging, committing, and status query against temporary test directories (`tempfile` crate).
2. **IPC Integration Tests:** Verify Tauri IPC command serialization and error mapping for uninitialized Git directories.
3. **Diff Generator Tests:** Verify structured JSON line diff output (`add`, `delete`, `context`).
4. **Frontend Type Check:** Verify React 19 Git Panel components compile cleanly against `@/types/generated/`.

---

## 7. Definition of Done

- [x] `GitRepositoryPort` trait and adapter implemented and tested in Rust.
- [x] Tauri IPC commands registered and `ts-rs` bindings exported.
- [x] React 19 visual Git Panel UI (Status, Commit, Branch Selector, Sync, JSON Diff Viewer) completed.
- [x] Automated `.gitignore` security exclusion template verified.
- [x] `cargo test` and `npm run build` pass 100% green.

## 8. Implementation Notes (as-built)

- **Adapter choice:** the user approved the subprocess implementation (`GitProcessAdapter`) instead of the `git2` crate — zero new dependencies (Rule 5), credentials handled by the system HTTPS helper / SSH agent, and no additional C dependency in the cross-compilation chain. Consequently `tempfile` was also skipped in favor of the established `temp_dir()` + `uuid` test pattern.
- JSON report DTO field names follow the spec's snake_case structs (no serde rename), so the Git panel consumes snake_case keys from the generated TypeScript bindings.
- The auto-generated `.gitignore` intentionally appears as an untracked file in the panel so users commit it and share the security exclusions with their team through the repository itself.
