# Software Design Document (SDD) & AI Agents Guidelines
**Project Name**: Postman Tauri Clone (TBD)
**Target Platform**: Desktop (Windows, macOS, Linux) cross-platform native executable.
**Environment**: WSL2 + Ubuntu running on Windows 11.

## 1. Project Overview
This project aims to build a modern, 100% desktop-native API client (a "Postman clone") from scratch. The application must be fast, lightweight, and focused on offline-first capabilities, utilizing an architecture that keeps the memory footprint extremely low (under ~80 MB) and application bundle sizes less than 50MB.

## 2. Technology Stack & Principles
- **Framework**: Tauri v2 (Rust backend, webview frontend)
- **Frontend**: React 18/19 + TypeScript + Vite
- **State Management**: TanStack Query (HTTP calls caching) & Zustand (Global state)
- **Styling**: Tailwind CSS
- **Code Editor / Syntax**: CodeMirror 6 or Monaco Editor
- **Storage Strategy**: Git-friendly `.json` or `.bru`-like plain text files. Secrets encrypted in a local vault.
- **Backend/Core Logic**: Rust (reqwest for HTTP, QuickJS for scripts)

### Development Principles
The project strictly implements **Clean Architecture** (Robert C. Martin) and **SOLID** principles:
1. **Domain Layer**: Contains pure, immutable business rules (`models.rs`, `errors.rs`, `value_objects.rs`). Independent of any external technologies or libraries.
2. **Application Layer**: Contains `Use Cases` and `Ports` (Interfaces/Traits). It coordinates the domain rules without knowing *how* the execution actually happens (i.e. depends on inversed dependencies).
3. **Infrastructure Layer**: Implements the Ports using concrete technologies (`reqwest` for HTTP, FileSystem, SQLite, rquickjs).
4. **Presentation Layer**: Thin API endpoints. Exclusively Tauri Commands (`#[tauri::command]`) mapping input to Use Cases.

**Agent Rule**: Under no circumstance should business logic be placed inside the Frontend (React). The React layer only dispatches `invoke()` events and handles visual representation.

## 3. Product Roadmap (Feature Prioritization)
### MVP (v1.0)
- **API Client**: HTTP methods (GET, POST, PUT, DELETE, PATCH).
- **Data Exchange**: Query params, complex headers, varied bodies (Raw JSON/XML, Form-data, Binary).
- **Collections & Environments**: Folder structures, collection sets, multiple environments with scoped variables.
- **Scripting & Automação**: Pre-request script & Post-response (Tests) using `pm.*` API syntax. Collection runner.
- **Security**: Local vault for storing tokens and passwords using strong encryption algorithms (e.g. cha-cha20).
- **Core UI**: Tabs interface, themes (Dark/Light), Command Palette capability.

### v1.5 - v2.0
- **Multi-protocol**: Support for GraphQL, WebSocket, gRPC, MQTT.
- **Offline Mock Server**: Read from collections to mock APIs locally.
- **Git Native Integration**: Connect a workspace directly to a Git remote repository.

## 4. Directory Structure (Clean Architecture & SOLID Mapped)

The folder hierarchy is fundamentally designed to enforce the Separation of Concerns and Dependency Inversion rules mandated by **Clean Architecture** (Robert C. Martin), **SOLID principles**, and **Clean Code** conventions. 

```text
postman-tauri-clone/
├── src/                          # FRONTEND (React)
│   ├── app/                      # Global config, Router, Providers
│   ├── features/                 # Feature Slice Design (Cohesive Context modules)
│   └── components/               # Atomic UI parts
├── src-tauri/                    # BACKEND (Rust)
│   └── src/
│       ├── domain/               # [Clean Arch: Entities] Pure enterprise logic, NO dependencies.
│       ├── application/          # [Clean Arch: Use Cases] Coordinates logic. Depends on abstractions (Ports, 'D' in SOLID).
│       ├── infrastructure/       # [Clean Arch: Frameworks/Adapters] Concrete implementations of Ports (Reqwest, IO). Open/Closed compliant.
│       └── presentation/         # [Clean Arch: Interface Adapters] Tauri endpoints mapping to Use Cases.
```

The strict boundary between `domain` -> `application` -> `infrastructure` guarantees high cohesiveness, straightforward testability, and isolated bug reproduction, directly applying the textbook knowledge from the cited literatures.

## 5. Development Guidelines for AI Agents
When generating new code or refactoring this project, all AI Agents must adhere to the following rules:
- **Dependency Rule**: Dependencies always point INWARDS. The Domain never imports Application, and Application never imports Infrastructure.
- **Interfaces First**: When needing a new external service, define a Port inside `application/ports/` before writing its concrete implementation in `infrastructure/`.
- **WSL Paths**: File paths strictly rely on Unix conventions in the command line since the workflow targets `./Ubuntu/...`.
- **Tests First**: Design code keeping `mocks` in mind. Pure functions in Rust should be tested immediately using isolated unit tests.
- **Git-friendly persistence**: Avoid binary blobs for configurations or requests. Use deterministic, ordered structural storage formats.
