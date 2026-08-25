# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-08-25

### Added — Initial Release

- **Zero Type-Drift Tauri IPC:** TypeScript bindings auto-generated from Rust domain models via `ts-rs`, with CI drift guard (`git diff --exit-code src/types/generated/`).
- **Headless CLI Runner (`tyny-cli`):** Run collections without the desktop GUI; JSON and JUnit XML reports with CI-friendly exit codes.
- **Git-Native Workspace Collaboration:** Visual source control panel inside the app — stage, commit, branch, push/pull to any Git remote, with interactive JSON diff viewer.
- **JavaScript Scripting Sandbox:** Pre-loaded libraries available via `require()` — Lodash, Day.js, Crypto-JS, UUID — executed in the QuickJS sandbox with the `pm.*` API.
- **Multi-Protocol Support:** Unified client for REST, GraphQL, WebSocket, and gRPC mock testing.
- **SpecHub:** OpenAPI 3.0 authoring, preview, and governance linting.
- **Security:** Local secret vault encrypted at rest with AES-256-GCM.

[1.0.0]: https://github.com/daniel-castilho/tyny-pulse/releases/tag/app-v1.0.0
