# Project Progress Tracker

This document is the global state map of the project. Any AI agent taking over the project must consult this file (together with `AGENTS.md`) to understand the current development phase.

## Current Stage: **Phase 18 (Headless Automation & CLI)** — In Progress 🚀

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

### Milestones Achieved (Zero Type-Drift Epic Complete ✅ - End-to-End Type Safety)
- [x] **IPC Surface Repair**: All 30+ Tauri commands compile, are registered, and match frontend payloads (`ws_*`, `ai_*`, `run_collection`, designs, monitors, sync, cookies).
- [x] **ts-rs Integration**: All IPC-crossing domain types derive `TS`; bindings generated into `src/types/generated/` by `cargo test export_ts_bindings` (39 types).
- [x] **Single Source of Truth**: Frontend consumes generated contracts via the `src/types/ipc.ts` barrel; hand-written duplicate interfaces removed from `workspaceStore.ts`.
- [x] **IPC Bug Fixes Surfaced By Typing**: `send_request` now returns a named `SendRequestOutput`; complete request literals (missing `name`/`description` fields fixed); environment fallbacks corrected to `variables: []`; runner/load-test payloads include `globals`/`sessionVars`.
- [x] **CI Type-Drift Guard**: `.github/workflows/ci.yml` regenerates bindings and fails when `src/types/generated/` is stale; also enforces clippy `-D warnings`, `npm run build`, and the architecture boundary grep.

---

## 🚀 Next Activities (Towards the Postman Killer)

### Phase 18: Headless Automation (The CLI) (NEXT FOCUS)
- [ ] **Tauri CLI Commands**: Add subcommands to the binary for running collections from the terminal.
- [ ] **JSON/JUnit Reporting**: Generate test reports for CI/CD pipelines.
- [ ] **GitHub Actions Integration**: Official automation template.

### Phase 19: Advanced Scripting & npm Integration
- [ ] **Dynamic Dependency Resolver**: Import npm packages inside scripts (expanded QuickJS sandbox).
- [ ] **In-App Package Manager**: Visual management of external libraries.

### Phase 16: Advanced Collaboration & Presence
- [ ] **Live Avatars/Presence**: Visual identification of active members in the workspace core.
- [ ] **Conflict Resolution UI**: Merge/diff system for simultaneous edits (Advanced CRDT).

---

### 📐 Architecture Directives (Attention!)
- [ ] **Strategy Pattern**: Whenever complex network/business `match` blocks accumulate, refactor into Traits + typed Enums for extensibility.

### Current Architecture
* **Frontend**: React 19 + Zustand + Monaco Editor + Lucide Icons + Framer Motion. Types imported from ts-rs generated bindings (`src/types/ipc.ts` barrel → `src/types/generated/`).
* **Backend**: Tauri v2 (Rust). Encryption via AES-256-GCM.
* **AI**: Gemini 1.5 Series (via logic layer).
* **Mock**: Axum Server embedded in Rust.
* **SpecHub**: OpenAPI Design & Governance Engine.
* **Sync**: Robust background with `Offline Queue` and connectivity detection.
* **Persistence**: Local-First via Fs repositories (Collection, Environment, Design, Globals).

---
_Last Updated: August 2026. Zero Type-Drift epic completed: end-to-end type safety across the Tauri IPC boundary. Focus now on terminal automation (CLI)._
