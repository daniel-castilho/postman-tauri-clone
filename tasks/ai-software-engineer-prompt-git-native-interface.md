# AI Software Engineer Prompt — Git-Native Interface (P3)

**Status:** Draft for implementation — Git-Native Workspace Collaboration Epic.
**Target:** Build an in-app visual Git control panel and backend Git repository adapter (`GitRepositoryPort` / `infrastructure/git/`) in Rust to enable cloud-free, zero-lock-in team collaboration on workspaces.
**Package / Scope:** `src-tauri/src/application/ports/git_repository.rs`, `src-tauri/src/infrastructure/git/`, `src-tauri/src/application/commands/git_tasks.rs`, `src/components/GitPanel/`

You implement the visual Git control panel and native Git operations engine for **Tyny Pulse** so that developers and teams can stage, commit, push, pull, switch branches, and view diffs for JSON workspace files directly within the application UI without relying on proprietary cloud services.

---

## Sources of Truth (Read in Order)

1. `AGENTS.md`
2. `docs/coding-standards.md` · `docs/testing-playbook.md` · `docs/data-model-decisions.md`
3. `tasks/git-native-interface-spec.md` — Technical Specification
4. `tasks/git-native-interface-backlog.md` — User Stories Map (S1–S9)
5. `tasks/git-native-interface-implementation-sequence.md` — Step-by-Step Execution Sequence
6. Reference: `src-tauri/src/domain/models.rs`, `src-tauri/src/application/ports/`, `src/store/workspaceStore.ts`

---

## Goal

Bring **Tyny Pulse** to 100% cloud-free team collaboration readiness with an embedded Git workflow engine:

- Define a clean domain port `GitRepositoryPort` in `application/ports/git_repository.rs` for querying branch status, uncommitted changes, staging, committing, pulling, pushing, and generating file diffs.
- Implement the concrete `Git2Adapter` (or CLI `GitProcessAdapter`) in `infrastructure/git/` ensuring safe, non-blocking execution over Tokio async threads.
- Expose Tauri IPC commands (`git_get_status`, `git_stage_file`, `git_unstage_file`, `git_commit`, `git_pull`, `git_push`, `git_list_branches`, `git_checkout_branch`, `git_get_file_diff`) returning strongly-typed DTOs annotated with `#[derive(TS)]`.
- Build a sleek React 19 visual Git Panel (`src/components/GitPanel/`) featuring:
  - Branch selector dropdown and active branch badge.
  - Staged and unstaged workspace file change list (collections, environments, settings).
  - Commit message editor with keyboard shortcut (`Ctrl+Enter` / `Cmd+Enter`).
  - Pull / Push action buttons with remote status indicators (ahead/behind counts).
  - Side-by-side JSON diff viewer for reviewing collection and environment mutations before committing.
- Ensure 100% local privacy: secrets in `.vault.enc` and machine-specific local settings remain excluded via default `.gitignore` rules.

---

## Non-Negotiable Rules

- **Clean Architecture Boundaries:** `GitRepositoryPort` must remain a pure Rust trait in `application/ports/`. The presentation layer and React frontend interact with Git capabilities strictly via Tauri IPC commands and generated DTO bindings.
- **Async Execution:** All Git I/O operations (fetch, pull, push, diff computation) must execute asynchronously in Rust (via Tokio blocking task pools) so the UI thread is never blocked during network pushes or large diff renders.
- **Vault & Credential Protection:** Never commit or push unencrypted secrets or `.vault.enc` master keys. Git staging must honor `.gitignore` patterns and warn users if unencrypted sensitive files are tracked.
- **Deterministic JSON Formatting:** Workspace JSON exports must maintain 2-space deterministic key ordering to ensure visual Git diffs remain minimal, readable, and conflict-free.
- **English Only:** All identifiers, attributes, IPC commands, UI labels, log messages, test functions, and commit messages must be in English.
- **Zero Behavior Change:** Core HTTP request execution, QuickJS scripting, and CLI execution functionality must remain 100% untouched.

---

## Definition of Done (Epic)

- [ ] `GitRepositoryPort` trait defined and implemented in `infrastructure/git/`.
- [ ] Tauri IPC commands for Git status, staging, commit, push, pull, branch checkout, and diff view registered and exported via `ts-rs`.
- [ ] React 19 visual Git Panel UI implemented and integrated into main layout.
- [ ] Side-by-side JSON diff viewer working smoothly for collection and environment files.
- [ ] Auto-generated `.gitignore` template for workspaces (excluding secrets/local overrides) verified.
- [ ] Rust unit & integration tests (`cargo test`) and frontend build (`npm run build`) pass 100% green.

Start at **Step 0** of `tasks/git-native-interface-implementation-sequence.md`. If any instruction or Git scope is unclear, **stop and ask**.
