# Data Model Decisions — Tyny Pulse

Record of data architecture decisions, storage strategies, and state representations in **Tyny Pulse**. Keep this file synchronized whenever workspace storage or data structures evolve.

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Local-First & Git-Native Workspaces

- **Source of Truth:** Plain-text, human-readable `.json` files stored directly on the user's local disk.
- **Git Compatibility:** To avoid merge conflicts and noisy diffs when workspaces are version-controlled via Git:
  - All JSON outputs are serialized with deterministic 2-space key ordering.
  - Floating-point timestamps use ISO-8601 strings (`2026-08-24T12:00:00Z`).
  - Request IDs use deterministic UUID v4 strings.

---

## 2. AES-256-GCM Encrypted Secret Vault

- **Source of Truth:** Encrypted `.vault.enc` files stored locally alongside the workspace.
- **Security Guarantee:**
  - Plaintext secrets (API keys, passwords, bearer tokens, private keys) are **never** written unencrypted to disk.
  - Master passphrase derives a 256-bit encryption key using PBKDF2 / Argon2.
  - Vault payload is encrypted using **AES-256-GCM** with hardware acceleration.
  - In-memory secret representations in Rust use zeroizing memory wrappers (`zeroize` crate) to scrub secrets from RAM upon drop.

---

## 3. Request & Response History Immutability

- **Source of Truth:** Local execution log store (`.history/` directory within workspace).
- **Immutability:**
  - Each request execution generates a unique `ExecutionId` snapshot containing the exact request sent (method, URL, headers, body) and the exact response received (status code, headers, body bytes, execution duration in ms).
  - History snapshots are read-only and capped by user-configurable retention limits (default: last 500 executions).

---

## 4. SpecHub (OpenAPI 3.0 / 3.1 AST & Linting)

- **Source of Truth:** Native YAML or JSON OpenAPI spec files.
- **In-Memory Representation:**
  - SpecHub parses specifications into a strongly-typed Abstract Syntax Tree (AST) in Rust.
  - Governance linting rules execute against the AST in real-time, returning line-level diagnostic markers (errors, warnings, hints) without mutating the underlying spec file on disk.

---

## 5. JavaScript Scripting Sandbox Scope (`pm.*` API)

- **Source of Truth:** QuickJS isolated runtime state.
- **State Mutation Isolation:**
  - Pre-request and test scripts execute inside an isolated QuickJS context.
  - Modifications to environment variables via `pm.environment.set("key", "value")` do not mutate global state directly during execution.
  - Script execution produces a structured `ScriptResult` delta containing environment updates, console logs, and assertion pass/fail results.
  - Environment updates are applied atomically to the active workspace environment only if the script completes successfully without uncaught exceptions.

---

## 6. Schema Rollout & Versioning Strategy

- All workspace JSON schemas include a root `schemaVersion: "1.0"` attribute.
- Backward compatibility is maintained via Rust domain migrations: when opening an older workspace, Tyny Pulse automatically migrates the JSON payload in-memory and prompts the user before writing the updated schema version back to disk.
