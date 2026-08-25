# Twelve-Factor Application — Compliance Matrix (Tyny Pulse)

The **Tyny Pulse** architecture adheres to the [Twelve-Factor App](https://12factor.net/) principles adapted for modern, high-performance desktop and local-first software.

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## Compliance Matrix

|   #    | Factor                  | Tyny Pulse Status | Architectural Compliance Notes                                                                                                                                   |
| :----: | :---------------------- | :---------------: | :--------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **1**  | **Codebase**            |   ✅ Compliant    | Single Git repository containing both the React 19 frontend (`src/`) and Rust backend core (`src-tauri/`).                                                       |
| **2**  | **Dependencies**        |   ✅ Compliant    | Fully declared and deterministically pinned via `Cargo.lock` (Rust crates) and `package-lock.json` (npm packages).                                               |
| **3**  | **Config**              |   ✅ Compliant    | Configuration and environments are strictly separated from code. Environment variables and encrypted secret vaults handle sensitive credentials.                 |
| **4**  | **Backing Services**    |   ✅ Compliant    | Target APIs, GraphQL servers, WebSocket endpoints, and gRPC hosts are treated as attached remote resources.                                                      |
| **5**  | **Build, Release, Run** |   ✅ Compliant    | Strict separation: Build (`npm run build`), Release (Tauri bundling, code signing, and auto-updater manifest creation), Run (native desktop executable).         |
| **6**  | **Processes**           |   ✅ Compliant    | The core execution engine is stateless. Workspaces are stored as local JSON files; in-memory UI state is managed by Zustand.                                     |
| **7**  | **Port Binding**        |   ✅ Compliant    | Completely self-contained desktop application. Vite dev server binds to a local port in dev mode; production releases run embedded static assets inside Webview. |
| **8**  | **Concurrency**         |   ✅ Compliant    | Concurrency scales via Tokio multi-threaded async tasks in Rust for request executions, collection runs, and load testing.                                       |
| **9**  | **Disposability**       |   ✅ Compliant    | Fast application startup times (< 500ms) and graceful shutdown handling via Tokio task cancellation.                                                             |
| **10** | **Dev/Prod Parity**     |   ✅ Compliant    | Development (`npm run tauri dev`) and Production (`npm run tauri build`) share the exact same Rust domain core and QuickJS sandbox.                              |
| **11** | **Logs**                |   ✅ Compliant    | Logs are treated as event streams sent to stdout or application log sinks. Raw secrets and decrypted vault contents are never logged.                            |
| **12** | **Admin Processes**     |   ✅ Compliant    | Administrative tasks (e.g., workspace schema migration, project renaming via `./rename_to_tyny_pulse.sh`) run as isolated, standalone scripts.                   |

---

## Core Guidelines to Maintain Compliance

- **Never Hardcode Secrets:** API keys, passwords, and tokens must always be resolved from environment variables or the AES-256 encrypted local vault.
- **Reproducible Builds:** Builds must be 100% reproducible from a fresh checkout using `npm install` and `cargo build`.
- **Local-First Isolation:** Local workspace files (`.json`) must remain clean, version-controllable, and free of machine-specific binary data.
