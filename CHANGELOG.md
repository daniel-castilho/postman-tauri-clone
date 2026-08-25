# Changelog

All notable changes to **Tyny Pulse** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App Identifier:** `ca.tyny.pulse`

---

## [Unreleased]

### Planned (P4 / P5 Epics)

- **Tokio Load Testing Engine:** Real-time RPS and latency percentile (p50, p95, p99) streaming charts in React 19 UI.
- **Ollama Local AI Provider:** 100% offline local LLM integration as a zero-cloud alternative to Gemini.
- **WASM Plugin System:** Extensible WebAssembly plugin architecture for custom authentication and protocol adapters.

---

## [1.0.0] - 2026-08-25

> **Note:** This is the initial public release of Tyny Pulse.  
> The entire codebase was developed on 2026-08-25 following strict Clean Architecture and SOLID principles with AI-assisted software engineering practices.

### Added

- **Zero Type-Drift IPC:** Automated TypeScript binding generation from 39 Rust IPC-crossing domain models via `ts-rs` + CI porcelain guard that fails builds on type drift.
- **Headless CLI (`tyny-cli`):** Standalone Rust binary providing headless collection execution, JSON and JUnit XML test reporters, environment overrides, and standardized process exit codes (`0`, `1`, `2`, `3`) for CI/CD pipelines.
- **Git-Native Collaboration Panel:** In-app source control interface (status, staging, commit, branch creation/switching, remote push/pull) using the system `git` executable, plus interactive side-by-side JSON diff viewer.
- **JavaScript Sandbox:** QuickJS-based execution engine with Postman-compatible `pm.*` API (`pm.test()`, `pm.environment.set()`, `pm.response`) + curated CommonJS `require()` support for Lodash (4.17.21), Day.js (1.11.13), Crypto-JS (4.2.0), and UUID (8.3.2).
- **Multi-Protocol Core:** Execution engine supporting REST (HTTP/1.1 & HTTP/2), GraphQL (query/mutation with variable editor), WebSocket (real-time frame inspector), and gRPC (mock server hub).
- **SpecHub:** OpenAPI 3.0 / 3.1 authoring and real-time governance linting engine implemented in Rust.
- **Encrypted Local Vault:** Military-grade AES-256-GCM local secret storage with Argon2 key derivation at rest.
- **Engineering Documentation:** Comprehensive architecture documentation including `AGENTS.md`, `docs/coding-standards.md`, `docs/testing-playbook.md`, `docs/release-runbook.md`, and `docs/twelve-factor.md`.
- **Open-Source Licensing:** Included official MIT License file (`LICENSE`).

### Changed

- **Branding Alignment:** Updated product name to **Tyny Pulse**, reverse domain app identifier to `ca.tyny.pulse`, and official links to `https://tyny.ca`.
- **Zustand Store Refactoring:** Migrated global React stores to curried `create<WorkspaceState>()()` syntax, removing all `any` type casts across the frontend codebase.
- **Documentation Accuracy:** Updated `README.md` to accurately reflect native CSS Modules & Glassmorphic styling (removing outdated Tailwind CSS references).

### Fixed

- **Tauri v2 Linux CI Pipelines:** Updated Ubuntu build workflows to WebKitGTK 4.1 / GTK 3 stack required by Tauri v2.
- **Sync Queue Contract:** Resolved payload key mismatches where sync queue sent camelCase keys instead of required snake_case contracts.
- **Environment State Mutation:** Fixed array spread mutation bug in environment variables editor preserving `current_value` semantics.
- **IPC Event Listener Cleanup:** Resolved event listener memory leaks during runner window mounting.

### Security

- **Vault Secret Protection:** Plaintext secrets never leave the `VaultPort` adapter in unencrypted storage and are never printed to logs or IPC outputs.
- **Automatic `.gitignore` Security Initializer:** Automated workspace protection excluding encrypted vaults (`*.vault.enc`), keys (`.vault.key`), and local environment overrides (`.env.local`) from Git staging.

### Known Limitations

- **gRPC Protocol:** Currently limited to mock server simulation in v1.0.0; native streaming gRPC client transport is planned for a future update.
- **AWS SigV4 Authentication:** AWS SigV4 auth header calculation includes placeholder signatures; use bearer token / basic auth for production APIs in v1.0.0.
- **AI Features:** Gemini AI explanation features require an external API key (local offline AI via Ollama is scheduled for P5).

---

[Unreleased]: https://github.com/daniel-castilho/tyny-pulse/compare/app-v1.0.0...HEAD
[1.0.0]: https://github.com/daniel-castilho/tyny-pulse/releases/tag/app-v1.0.0
