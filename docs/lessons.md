# Lessons Learned — Tyny Pulse

Practical findings from developing, testing, and hardening **Tyny Pulse**. Add to this file whenever a non-obvious failure, compiler issue, or desktop runtime quirk costs real debugging time.

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Debugging Discipline (How to Attack Failures)

- **Research Beats Trial-and-Error:** When a failure is not instantly explainable from local evidence, search primary sources FIRST (exact error text + crate name + version).
- **Triage Easiest First:** When faced with multiple compiler or build failures, build a full error inventory (`cargo check 2>&1 | grep error`), fix independent syntax/type mismatches first, then re-run to expose deep architectural errors.
- **Tauri IPC Serialization Boundary:** Tauri IPC passes data between Rust and Webview via JSON serialization. Any Rust domain struct exposed over `#[tauri::command]` must implement `serde::Serialize` and `serde::Deserialize`. Never expose raw pointers or non-serializable Rust traits across the IPC boundary.

---

## 2. Desktop Runtime & Tauri v2

- **Linux / WebKitGTK System Dependencies:** On Linux and WSL2 environments, Tauri v2 requires `libwebkit2gtk-4.1-dev`, `build-essential`, `libssl-dev`, and `libayatana-appindicator3-dev`. Ensure system packages are installed before running `npm run tauri dev`.
- **Asynchronous Command Execution:** Tauri commands that execute long-running tasks (e.g. load testing or collection execution) must be marked `async fn` and use Tokio async channels or Tauri events (`app_handle.emit()`) to stream progress back to the frontend without blocking the UI thread.

---

## 3. QuickJS JavaScript Sandbox

- **Cross-Compilation C Bindings:** QuickJS relies on underlying C code compilation. When building production binaries for Windows (`x86_64-pc-windows-msvc`) or macOS (`aarch64-apple-darwin`), ensure MSVC or Xcode toolchains are correctly configured in CI workers.
- **Memory Limits in Sandbox:** Limit memory allocation for QuickJS runtime instances to prevent runaway user scripts from consuming desktop system RAM.

---

## 4. Pending Lessons & Future Log

*(This section is updated as new debugging lessons and infrastructure findings emerge during development.)*
