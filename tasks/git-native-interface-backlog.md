# Git-Native Interface — Backlog (P3)

**Companions:** `git-native-interface-spec.md` · `git-native-interface-implementation-sequence.md` · `ai-software-engineer-prompt-git-native-interface.md`  
**Epic Goal:** Build an in-app visual Git control panel and backend Git repository adapter in Rust to enable cloud-free, zero-lock-in team collaboration on workspaces.

**MVP Scope:** Stories S1–S9

---

## Story Map

```text
PORT & ADAPTER FOUNDATION
S1 Define GitRepositoryPort trait and domain DTOs in Rust
S2 Implement concrete Git adapter in infrastructure/git/ using git2 / subprocess
S3 Add automatic .gitignore security initializer for workspaces

TAURI IPC & TYPE BINDINGS
S4 Implement Tauri IPC commands for Git operations in application/commands/git_tasks.rs
S5 Export TypeScript bindings for Git DTOs via ts-rs export_ts_bindings test

VISUAL UI COMPONENTS (REACT 19)
S6 Build visual Git Panel layout (Branch selector, Status list, Commit editor)
S7 Build interactive JSON Diff Viewer modal for staged and unstaged changes
S8 Integrate Push, Pull, and Remote Sync actions with status indicators

VERIFICATION & CI
S9 Add integration tests for Git adapter and validate full regression suite
```

---

## Stories Breakdown

| ID | Story Title | Priority | Target Modules / Components | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **S1** ✅ | Define `GitRepositoryPort` trait and domain DTOs in Rust | Must | `src-tauri/src/application/ports/git_repository.rs`, `domain/models.rs` | Interfaces for status, stage, commit, push, pull, branch, diff |
| **S2** ✅ | Implement concrete Git adapter in `infrastructure/git/` | Must | `src-tauri/src/infrastructure/git/git_process_adapter.rs` | User-approved subprocess implementation (zero new crates; system credentials) |
| **S3** ✅ | Add automatic `.gitignore` security initializer for workspaces | Must | `src-tauri/src/infrastructure/git/git_initializer.rs` | Automatically ignores `.vault.enc`, `.vault.key`, `.env.local` |
| **S4** ✅ | Implement Tauri IPC commands in `application/commands/git_tasks.rs` | Must | `src-tauri/src/application/commands/git_tasks.rs`, `main.rs` | Expose `git_get_status`, `git_commit`, `git_push`, `git_pull`, `git_diff` |
| **S5** ✅ | Export TypeScript bindings for Git DTOs via `ts-rs` | Must | `src-tauri/src/application/commands/export_ts_bindings.rs` | Generates TS interfaces in `src/types/generated/` |
| **S6** ✅ | Build visual Git Panel layout (Branch selector, Status list, Commit editor) | Must | `src/components/GitPanel/GitPanel.tsx`, `Sidebar.tsx` | UI panel with status badges, staging toggles, commit box |
| **S7** ✅ | Build interactive JSON Diff Viewer modal | Must | `src/components/GitPanel/GitDiffViewer.tsx` | Side-by-side or unified line-by-line diff view for JSON files |
| **S8** ✅ | Integrate Push, Pull, and Remote Sync actions | Must | `src/components/GitPanel/GitSyncHeader.tsx` | Fetch/Push/Pull buttons with ahead/behind commit indicators |
| **S9** ✅ | Add integration tests for Git adapter & verify regression suite | Must | `src-tauri/tests/git_adapter.rs`, `docs/testing-playbook.md` | Test git status, stage, commit against temporary git repo |

---

## Definition of Done (Epic)

- [x] S1–S9 completed and verified.
- [x] In-app Git Panel displays active repository status, branch, staged/unstaged changes.
- [x] Staging, committing, pushing, and pulling function cleanly from the desktop UI.
- [x] Interactive JSON Diff Viewer displays readable additions and deletions.
- [x] Automatic `.gitignore` prevents unencrypted secret vaults from being committed.
- [x] `cargo test` and `npm run build` pass 100% green.
