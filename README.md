# ⚡ Tyny Pulse

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D5?style=for-the-badge&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2021-000000?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6?style=for-the-badge&logo=typescript&logoColor=white)
![Vite](https://img.shields.io/badge/Vite-7.0-646CFF?style=for-the-badge&logo=vite&logoColor=white)
![Domain](https://img.shields.io/badge/Domain-tyny.ca-8A2BE2?style=for-the-badge)

**Tyny Pulse** is an ultra-fast, local-first, cross-platform API client built with **Tauri v2**, **Rust**, and **React 19**. Designed as a lightweight, memory-efficient alternative to traditional resource-heavy API clients, Tyny Pulse delivers a zero-latency desktop experience, multi-protocol execution, built-in OpenAPI SpecHub, AES-256 encrypted local vaults, and Git-native JSON workspaces.

Official Domain: [https://tyny.ca](https://tyny.ca)

---

## Table of Contents

- [Key Features](#key-features)
- [Tech Stack](#tech-stack)
- [Architecture & Design Rules](#architecture--design-rules)
- [Project Structure](#project-structure)
- [Requirements](#requirements)
- [Getting Started](#getting-started)
- [Commands](#commands)
- [Automation & JS Scripting Sandbox](#automation--js-scripting-sandbox)
- [Protocol Support](#protocol-support)
- [Security & Local-First Philosophy](#security--local-first-philosophy)
- [Roadmap](#roadmap)
- [Documentation & License](#documentation--license)

---

## Key Features

- ⚡ **Rust-Powered Performance:** Ultra-low memory footprint (~50-80 MB RAM) with near-instant application startup times.
- 📂 **Local-First & Git-Native:** Store workspaces, collections, and environments in readable, version-controlled JSON files on your local disk.
- 📑 **Multi-Protocol Hub:** Unified support for REST, GraphQL, WebSockets, and gRPC mock testing.
- 📜 **SpecHub (API Design):** Author, preview, and validate OpenAPI 3.0/3.1 specifications with real-time governance linting.
- 🔐 **Military-Grade Security:** Local secret storage encrypted at rest with AES-256-GCM.
- 📜 **JavaScript Automation Sandbox:** Execute pre-request logic, post-response assertions, and dynamic environment manipulation using the familiar `pm.*` API powered by QuickJS.
- 🚀 **Built-in Load Testing:** Multi-threaded stress testing engine powered by Rust Tokio.
- 🔄 **Resilient Offline Sync:** Background synchronization queue designed for unstable network conditions.
- 🧬 **Zero Type-Drift IPC:** Every Tauri command payload is derived from the Rust domain models; TypeScript bindings are generated into `src/types/generated` by `cargo test` and a CI job fails the build if they ever drift.
- 🖥️ **Headless CLI:** Run collections from the terminal (`tyny-pulse run collection.json`) with `pm.*` assertions, JSON/JUnit reports, and pipeline-friendly exit codes — see [CLI Usage](#cli-usage).
- 💎 **Elite UI/UX:** Glassmorphic interface with Command Palette (`Ctrl+P` / `Cmd+P`), Framer Motion transitions, and dynamic Dark/Light themes.

---

## Tech Stack

| Category | Technology |
| :--- | :--- |
| **Core & Desktop Engine** | ![Tauri](https://img.shields.io/badge/Tauri_v2-24C8D5?style=for-the-badge&logo=tauri&logoColor=white) ![Rust](https://img.shields.io/badge/Rust_Tokio-000000?style=for-the-badge&logo=rust&logoColor=white) |
| **HTTP & Runtime Services** | `reqwest` (HTTP Client), `QuickJS` (JS Sandbox), `tokio` (Async Multithreading) |
| **Frontend Framework** | ![React](https://img.shields.io/badge/React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black) ![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white) |
| **Build & Styling** | ![Vite](https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white) Tailwind CSS / CSS Modules, Framer Motion, Lucide Icons |
| **State Management** | ![Zustand](https://img.shields.io/badge/Zustand-443E38?style=for-the-badge&logo=react&logoColor=white) |
| **Security** | AES-256-GCM encryption at rest |
| **Target OS** | ![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white) ![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white) ![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black) |

---

## Architecture & Design Rules

Tyny Pulse strictly implements **Clean Architecture** (Robert C. Martin) and **SOLID principles**, enforcing a strict boundary between business logic and framework presentation layers.

```
tyny-pulse/
├── src/                          # FRONTEND (React 19 + TypeScript)
│   ├── components/               # Atomic UI & Layout components
│   ├── store/                    # Lean state management (Zustand)
│   ├── types/                    # Domain models & TypeScript interfaces
│   ├── App.tsx                   # Core App entry & router layout
│   └── main.tsx                  # React mounting point
│
└── src-tauri/                    # BACKEND & DESKTOP (Rust + Tauri v2)
    ├── Cargo.toml                # Rust crate configuration
    ├── tauri.conf.json           # Application manifest (App ID: ca.tyny.pulse)
    └── src/
        ├── domain/               # Pure business entities & port trait definitions
        ├── application/          # Use cases, command handlers & orchestration
        ├── infrastructure/       # Adapters (Reqwest HTTP, QuickJS sandbox, AES Vault)
        └── main.rs               # Desktop application bootstrap & command bindings
```

### Boundary Rules
- **Domain & Application layers** in Rust never import infrastructure code or web framework bindings.
- **Frontend state** stays decoupled from Tauri IPC commands via clean service adapters.
- **Zero-Cloud Lock-In:** All workspace states reside locally in user-controlled JSON files.

---

## Requirements

Ensure you have the following installed on your development environment:

- **Node.js**: v18.0.0 or higher
- **Rust**: 1.75.0 or higher (`rustup`)
- **System Dependencies** (Linux / WSL2):
  ```bash
  sudo apt update
  sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```

---

## Getting Started

### 1. Clone the repository
```bash
git clone https://github.com/daniel-castilho/postman-tauri-clone.git tyny-pulse
cd tyny-pulse
```

### 2. Install dependencies
```bash
npm install
```

### 3. Run in Development Mode
```bash
npm run tauri dev
```

### 4. Build for Production
```bash
npm run tauri build
```
The compiled, native installers (`.exe`, `.msi`, `.dmg`, `.AppImage`, `.deb`) will be generated inside `src-tauri/target/release/bundle/`.

---

## CLI Usage

The same binary runs collections headlessly — no window, no GUI dependencies. Perfect for CI/CD pipelines:

```bash
# Build once (or use the desktop installer's bundled binary)
cargo build --release --manifest-path src-tauri/Cargo.toml

BIN=src-tauri/target/release/tyny-pulse

$BIN run collections/smoke.json \
    --env environments/staging.json \
    --var baseUrl=https://staging.api.example.com \
    --report report.junit
```

| Flag | Purpose |
| :--- | :--- |
| `--env <path>` | Load an environment JSON file |
| `--globals <path>` | Load global variables JSON |
| `--var <key=value>` | Override/inject an environment variable (repeatable) |
| `--report <path>` | Write a report file (`.json` or `.xml`/`.junit` decides the writer) |
| `--format json\|junit` | Force the writer when the extension is unknown |

Exit codes: `0` all tests passed · `1` test failures · `2` usage/input error · `3` domain error.

A ready-to-copy GitHub Actions workflow lives in [`docs/examples/github-actions-collection-run.yml`](docs/examples/github-actions-collection-run.yml).

---

## Commands

| Purpose | Command |
| :--- | :--- |
| **Run Dev Server (Vite + Tauri)** | `npm run tauri dev` |
| **Run Web Preview Only** | `npm run dev` |
| **Type Check & Web Build** | `npm run build` |
| **Build Native Desktop Binary** | `npm run tauri build` |
| **Run Rust Backend Checks** | `cd src-tauri && cargo check` |
| **Run Rust Tests** | `cd src-tauri && cargo test` |

---

## Automation & JS Scripting Sandbox

Tyny Pulse includes an isolated **JavaScript Execution Engine** powered by QuickJS. You can automate test workflows, validate payloads, and manipulate environment variables using the `pm.*` API.

### Pre-request Script Example
```javascript
// Generate dynamic tokens or timestamps before sending the request
const timestamp = new Date().toISOString();
pm.environment.set("request_time", timestamp);

const randomId = "test_" + Math.floor(Math.random() * 10000);
pm.environment.set("test_correlation_id", randomId);
```

### Response Test Script Example
```javascript
// Post-request assertion workflow
pm.test("Status code is 200 OK", () => {
    expect(pm.response.status).to.equal(200);
});

pm.test("Response time is under 200ms", () => {
    expect(pm.response.responseTime).to.be.below(200);
});

pm.test("Validate User Payload", () => {
    const json = pm.response.json();
    expect(json.id).to.be.a("number");
    expect(json.email).to.include("@");
    
    // Save token for subsequent requests in collection
    pm.environment.set("auth_token", json.token);
});
```

---

## Protocol Support

```
                  ┌─────────────────────────────────────────┐
                  │               TYNY PULSE                │
                  └────────────────────┬────────────────────┘
                                       │
         ┌─────────────────┬───────────┴─────────┬──────────────────┐
         │                 │                     │                  │
   ┌─────▼─────┐     ┌─────┴─────┐         ┌─────▼─────┐      ┌─────▼─────┐
   │   REST    │     │  GraphQL  │         │ WebSocket │      │   gRPC    │
   │ HTTP/1,2  │     │ Query/Mut │         │ Real-time │      │ Mock Hub  │
   └───────────┘     └───────────┘         └───────────┘      └───────────┘
```

- **REST API:** GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD with full header and body controls.
- **GraphQL:** Query and Mutation execution with schema introspection and variable editor.
- **WebSocket:** Real-time bidirectional connection testing with message history and frame inspector.
- **gRPC (Mock Hub):** Service definition testing with protobuf support.

---

## Security & Local-First Philosophy

- **AES-256-GCM Encryption:** Credentials, tokens, and environment secrets are encrypted locally using hardware-backed key derivation.
- **Offline Capability:** Network loss does not prevent you from viewing, editing, or organizing your collections.
- **Git Native:** Workspaces are stored as structured JSON files, making team collaboration via Git branches, PRs, and code reviews seamless.

---

## Roadmap

- [x] Multi-protocol core execution (REST, GraphQL, WS, gRPC)
- [x] Clean Architecture with Rust engine + React 19 UI
- [x] JS Sandbox (`pm.*` API compatibility)
- [x] AES-256 encrypted local vaults
- [x] SpecHub OpenAPI 3.0 / 3.1 real-time linter
- [x] Zero Type-Drift IPC: TypeScript bindings auto-generated from Rust domain models via `ts-rs`, enforced by CI
- [x] Headless CLI runner (`tyny-pulse run`) with JSON/JUnit reports and CI exit codes
- [ ] Collection Runner with exportable HTML/Markdown reports
- [ ] Cloud sync opt-in via `tyny.ca` relay

---

## Documentation & License

| Document | Purpose |
| :--- | :--- |
| [`README.md`](README.md) | Project overview, architecture, setup, and scripting guide |
| [`AGENTS.md`](AGENTS.md) | Guidelines for AI agents and human contributors |
| [`docs/progress.md`](docs/progress.md) | Feature implementation roadmap & architectural milestones |
| [`rename_to_tyny_pulse.sh`](rename_to_tyny_pulse.sh) | Automated script to rename package references across the codebase |

Designed & Maintained with ❤️ for [Tyny.ca](https://tyny.ca).
