# Git-Native Interface — Implementation Sequence (P3)

**Companions:** `git-native-interface-spec.md` · `git-native-interface-backlog.md` · `ai-software-engineer-prompt-git-native-interface.md`  
**Rule:** Finish each step's "Done when" before moving to the next. Do not invent scope.

---

## Step 0 — Analysis & Dependency Setup

1. Check if `git2` crate (libgit2 bindings) or process execution adapter fits the target platform requirements.
2. Add dependencies to `src-tauri/Cargo.toml` if using `git2`:
   ```toml
   [dependencies]
   git2 = "0.19"
   ```
3. Verify `cargo check --manifest-path src-tauri/Cargo.toml` compiles cleanly.

**Done when:** Git library dependency is integrated and verified in Cargo.toml.

---

## Step 1 — Domain DTOs & Port Definition

1. Define Git DTO structs in `src-tauri/src/domain/models.rs`:
   - `GitFileStatusType` (Untracked, Modified, Added, Deleted, Conflicted)
   - `GitFileChangeDto` (`path`, `status`, `is_staged`)
   - `GitStatusSummaryDto` (`is_repository`, `current_branch`, `branches`, `ahead_count`, `behind_count`, `files`)
   - `GitDiffChunkDto` & `GitFileDiffDto`
2. Define trait `GitRepositoryPort` in `src-tauri/src/application/ports/git_repository.rs`.
3. Annotate all DTOs with `#[derive(TS)]` and export paths to `../../src/types/generated/`.

**Done when:** `GitRepositoryPort` trait and DTOs compile cleanly in Rust.

---

## Step 2 — Concrete Git Adapter & Security Initializer

1. Create module `src-tauri/src/infrastructure/git/mod.rs` and `git2_adapter.rs`.
2. Implement `GitRepositoryPort` for `Git2Adapter`:
   - `get_status`: queries repository status and ahead/behind counts.
   - `stage_path` / `unstage_path`: modifies index.
   - `commit`: creates commit object on active branch.
   - `push` / `pull`: handles network remotes.
   - `get_diff`: generates line-by-line diff chunks for JSON files.
3. Implement `git_initializer.rs`: checks for `.gitignore` in workspace root and ensures `.vault.enc`, `.vault.key`, `.env.local` are listed.
4. Add unit tests in `src-tauri/tests/git_adapter.rs` using `tempfile::tempdir()`.

**Done when:** `cargo test` passes Git adapter unit and integration tests against temporary repositories.

---

## Step 3 — Tauri IPC Commands Registration

1. Create command handler module `src-tauri/src/application/commands/git_tasks.rs`.
2. Implement Tauri commands:
   - `git_get_status`
   - `git_stage_file`
   - `git_unstage_file`
   - `git_commit`
   - `git_push`
   - `git_pull`
   - `git_checkout_branch`
   - `git_get_file_diff`
3. Register commands in `src-tauri/src/main.rs`.
4. Update `src-tauri/src/application/commands/export_ts_bindings.rs` and execute `cargo test export_ts_bindings`.

**Done when:** TypeScript definitions for Git DTOs are generated in `src/types/generated/` and IPC commands are registered.

---

## Step 4 — Visual Git Panel UI Components (React 19)

1. Create component directory `src/components/GitPanel/`.
2. Build `GitPanel.tsx`:
   - Branch header with active branch name and branch switcher modal.
   - Sync controls (Push, Pull, Fetch) with ahead/behind counters.
   - Staged and Unstaged file lists with status badges.
   - Commit message box with `Ctrl+Enter` commit shortcut.
3. Integrate `GitPanel` trigger icon into the workspace sidebar (`Sidebar.tsx`).
4. Connect Git Panel state to Zustand workspace store (`src/store/workspaceStore.ts`).

**Done when:** Git Panel renders in the UI and communicates with Tauri IPC commands.

---

## Step 5 — Visual JSON Diff Viewer Modal

1. Build `GitDiffViewer.tsx`:
   - Modal displaying side-by-side or unified line diff for selected JSON file.
   - Highlight additions (green) and deletions (red) based on `GitFileDiffDto`.
2. Connect file click events in `GitPanel` to open `GitDiffViewer`.

**Done when:** Clicking a modified workspace JSON file opens the visual diff modal.

---

## Step 6 — Final Verification & Smoke Path

1. Run full build & test suite:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   npm run build
   ```
2. Run desktop app in dev mode (`npm run tauri dev`).
3. Execute end-to-end smoke path:
   - Open a workspace backed by a Git repo.
   - Edit a request in a collection → verify file appears under Unstaged changes.
   - Click file → verify JSON Diff Viewer displays modified properties.
   - Stage file → type commit message → click Commit.
   - Verify commit succeeds and status refreshes.
4. Update `docs/progress.md`, `README.md`, and `docs/testing-playbook.md`.

**Done when:** Epic Definition of Done is fully met.

---

## Smoke Path

1. UI Git Panel displays active repository status and branch.
2. Modifying a collection file reflects immediately under Unstaged changes.
3. Clicking file opens `GitDiffViewer` showing clean line diffs.
4. Stage and commit operations complete successfully without leaving the app.
5. Auto-generated `.gitignore` prevents `.vault.enc` from appearing in unstaged files.
