# Project Progress Tracker

This document is the global state map of the project. Any AI agent taking over the project must consult this file (together with `AGENTS.md`) to understand the current development phase.

## Current Stage: **Phase 16 (Advanced Collaboration & Presence)** — Next Focus

---

### Milestones Achieved (Git-Native Interface Complete ✅ - P3 Epic)

- [x] **`GitRepositoryPort`**: Pure async Rust port (`application/ports/git_repository.rs`) for status, staging, commit, push/pull, branch checkout and file diffs.
- [x] **`GitProcessAdapter`**: Subprocess-based adapter over the system `git` executable — zero new crates (Rule 5), credentials resolved by the user's own HTTPS helper/SSH agent, all blocking work on `tokio::task::spawn_blocking`.
- [x] **Security Initializer**: Idempotent `.gitignore` generator keeps `*.vault.enc`, `.vault.key`, `.env.local`, `script-libraries-local.json` out of tracked changes; refreshed on every status read.
- [x] **IPC Surface**: `git_get_status`, `git_list_branches`, `git_stage_file`, `git_unstage_file`, `git_stage_all`, `git_commit`, `git_push`, `git_pull`, `git_checkout_branch`, `git_get_file_diff`; five Git DTOs exported via the zero type-drift pipeline.
- [x] **Visual Git Panel** (`src/components/GitPanel/`): branch badge + switcher/creation, ahead/behind counters (↑↓), staged/unstaged lists with status badges, stage-all, commit editor with Ctrl+Enter, unified JSON diff viewer modal; triggered from the sidebar.
- [x] **Integration Tests**: Real temporary repositories validate lifecycle, vault-file exclusion, unborn-HEAD branches and non-repository workspaces (86 total tests green).

### Milestones Achieved (Phase 19 Complete ✅ - Advanced Scripting & npm Integration)

- [x] **`require()` in Sandbox Scripts**: Curated library registry embedded at compile time (`include_str!`) and resolved by a CommonJS-shaped `require('name')` shim inside the QuickJS setup script.
- [x] **Bundled MIT Libraries**: lodash 4.17.21, dayjs 1.11.13, crypto-js 4.2.0, uuid 8.3.2 (`src-tauri/assets/script-libs/` + THIRD-PARTY-NOTICES.md); each bundle smoke-tested in-sandbox (incl. NIST SHA-256 vector).
- [x] **In-App Package Manager**: Workspace Settings → Script Libraries section toggles libraries per workspace; persisted to Git-friendly `script-libraries.json`, re-read on every execution.
- [x] **WebCrypto Polyfill**: `crypto.getRandomValues` shim (Math.random fallback) unblocks uuid@8 inside QuickJS; documented as non-cryptographic.
- [x] **New IPC Surface**: `configure_script_engine`, `list_script_libraries`, `set_script_library_enabled`; `ScriptLibraryInfo` binding exported via the zero type-drift pipeline.

### Milestones Achieved (Phase 18 Complete ✅ - Headless Automation & CLI)

- [x] **Headless CLI Mode**: `tyny-cli run <collection.json>` executes collections from the terminal without booting the desktop shell (`presentation/cli.rs` branches before the Tauri builder; dedicated `src-tauri/src/bin/tyny-cli.rs` target without the Windows GUI subsystem attribute).
- [x] **JSON/JUnit Reporting**: Machine-readable reports via `infrastructure/reporting/` (spec §5 envelope: schema version, request/assertion summary, `durationMs`; JUnit XML with `errors`, `classname`, root `time`) — no new dependencies.
- [x] **Pipeline-Friendly Exit Codes**: 0 = all tests passed, 1 = test failures, 2 = usage/input error, 3 = domain error.
- [x] **CLI Ergonomics**: short/long flags (`-e/--env`, `-g/--globals`, `-v/--var key=value`, `-r/--report`, `-f/--format`), report format inference from file extension.
- [x] **GitHub Actions Integration**: Example workflow at `.github/workflows/tyny-cli-ci.yml` (workflow_dispatch); deterministic fixture at `src-tauri/tests/fixtures/sample_collection.json`.
- [x] **E2E Verified**: Integration tests spin a local mock HTTP server and validate nested folder traversal, `pm.*` assertions, both report writers and all exit code paths; the desktop `tyny-pulse` binary keeps full CLI parity.

---

### Milestones Achieved (Phases 8-14 Complete ✅ - Base Platform)

- [x] **Multi-Protocol**: GraphQL, WebSocket, gRPC (Mock Hub).
- [x] **Cookie Manager**: Automatic session persistence.
- [x] **AI-Native**: Test Generator and Explanations via Gemini.
- [x] **Load Testing Engine**: Multi-threaded tests in Rust (Tokio).
- [x] **Collaborative Sync**: Real-time sync via Tauri Events with `SyncQueue`.
- [x] **Workspace Member Management**: Access management with granular control.

### Milestones Achieved (Phase 15 Complete ✅ - DX & Power Tools)

- [x] **Global Command Palette (Ctrl + P)**: Instant workspace-wide search (Requests, Envs, Actions).
- [x] **Keyboard Shortcuts Engine**: Premium shortcuts for `Send`, `Save` and Navigation.
- [x] **Quick Environment Switcher**: Instant switching via keyboard (Ctrl + 1-9).

### Milestones Achieved (Phase 17 Complete ✅ - Enterprise Security)

- [x] **Local Secrets Encryption (AES-256-GCM)**: Military-grade secret protection on disk via Rust.
- [x] **Offline Sync Queue**: Extreme resilience for syncing under unstable networks.
- [x] **Transparent Security**: Automatic and secure decryption of local variables.

### Milestones Achieved (Phase 20 Complete ✅ - SpecHub / API Design)

- [x] **API Design Hub**: Dedicated space for authoring OpenAPI 3.0/3.1 specifications.
- [x] **Governance Linter**: Design-standard validation in real time in the backend (Rust).
- [x] **Unified Context Switching**: Hybrid interface for Design and Execution.

### Milestones Achieved (Load Testing Engine Epic Complete ✅ - P4)

- [x] **Tokio Engine (`LoadTestService`)**: 1–500 virtual users as lightweight Tokio tasks with ramp-up staggering and per-request timeout (`application/services/load_test_service.rs`).
- [x] **Lock-Free Aggregation**: Workers push `Sample { duration_ms, status_code, bytes }` into a bounded `mpsc` channel; a single-writer aggregator owns all metrics — zero mutexes on the hot path.
- [x] **Real-Time Streaming**: `load_test_progress` Tauri events every 200ms carrying RPS (smoothed), active VUs, throughput, status-code breakdown and nearest-rank percentiles (p50/p90/p95/p99 over sorted sample windows).
- [x] **Graceful Cancellation**: `stop_load_test` flips a `tokio::sync::watch` flag selected upon by workers and aggregator — verified by test to halt in < 100ms.
- [x] **IPC Surface**: `start_load_test`, `stop_load_test`, `get_load_test_status` registered in `main.rs`; `LoadTestConfigDto`, `LoadTestProgressEventDto`, `LatencyPercentilesDto`, `StatusCodeCountDto` exported via ts-rs.
- [x] **React Panel (`src/components/LoadTestingPanel/`)**: target request picker, VUs slider (1–500), duration/ramp-up/timeout controls, live metric cards, dependency-free animated SVG charts (RPS line, p50/p95/p99 multi-line, status donut) and JSON/Markdown report export.
- [x] **Legacy Preserved**: blocking `run_load_test` contract untouched ("Zero Behavior Change"); new consumers use the streaming commands.

### Milestones Achieved (Zero Type-Drift Epic Complete ✅ - End-to-End Type Safety)
- [x] **IPC Surface Repair**: All 30+ Tauri commands compile, are registered, and match frontend payloads (`ws_*`, `ai_*`, `run_collection`, designs, monitors, sync, cookies).
- [x] **ts-rs Integration**: All IPC-crossing domain types derive `TS`; bindings generated into `src/types/generated/` by `cargo test export_ts_bindings` (39 types).
- [x] **Single Source of Truth**: Frontend consumes generated contracts via the `src/types/ipc.ts` barrel; hand-written duplicate interfaces removed from `workspaceStore.ts`.
- [x] **IPC Bug Fixes Surfaced By Typing**: `send_request` now returns a named `SendRequestOutput`; complete request literals (missing `name`/`description` fields fixed); environment fallbacks corrected to `variables: []`; runner/load-test payloads include `globals`/`sessionVars`.
- [x] **CI Type-Drift Guard**: `.github/workflows/ci.yml` regenerates bindings and fails when `src/types/generated/` is stale; also enforces clippy `-D warnings`, `npm run build`, and the architecture boundary grep.

---

## 🚀 Next Activities (Towards the Postman Killer)

### Phase 18: Headless Automation (The CLI) ✅ (Completed — see milestones above)

### Phase 19: Advanced Scripting & npm Integration ✅ (Completed — see milestones above)

- [ ] **Dynamic Dependency Resolver (runtime download)**: Fetch arbitrary npm packages on demand — deferred; needs new crates (`tar`/`flate2`) and network host approval (Rule 5).
- [ ] **Expanded curated registry**: ajv, moment-timezone and similar candidates.

### Phase 16: Advanced Collaboration & Presence (NEXT FOCUS)

- [ ] **Live Avatars/Presence**: Visual identification of active members in the workspace core.
- [ ] **Conflict Resolution UI**: Merge/diff system for simultaneous edits (Advanced CRDT).

---

### 📐 Architecture Directives (Attention!)

- [ ] **Strategy Pattern**: Whenever complex network/business `match` blocks accumulate, refactor into Traits + typed Enums for extensibility.

### Current Architecture

- **Frontend**: React 19 + Zustand + Monaco Editor + Lucide Icons + Framer Motion. Types imported from ts-rs generated bindings (`src/types/ipc.ts` barrel → `src/types/generated/`).
- **Backend**: Tauri v2 (Rust). Encryption via AES-256-GCM.
- **AI**: Gemini 1.5 Series (via logic layer).
- **Mock**: Axum Server embedded in Rust.
- **SpecHub**: OpenAPI Design & Governance Engine.
- **Sync**: Robust background with `Offline Queue` and connectivity detection.
- **Persistence**: Local-First via Fs repositories (Collection, Environment, Design, Globals).

---

_Last Updated: August 2026. Git-Native Interface (P3) completed: in-app Git panel over `GitRepositoryPort`/`GitProcessAdapter` with secure `.gitignore` generation and typed IPC. Next focus: Phase 16 (Advanced Collaboration & Presence)._
