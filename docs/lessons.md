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

_(This section is updated as new debugging lessons and infrastructure findings emerge during development.)_

---

## 5. CI/CD Lessons (GitHub Actions)

- **Tauri v2 requires the WebKitGTK 4.1 stack on Ubuntu CI runners:** installing the Tauri v1-era `libwebkit2gtk-4.0-dev` makes `javascriptcore-rs-sys` / `soup3-sys` build scripts fail with "The file `<lib>.pc` needs to be installed and PKG_CONFIG_PATH..." errors. The correct set (Ubuntu >= 22.04) is `libwebkit2gtk-4.1-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `pkg-config`. Sources: tauri-apps/tauri-action#720, tauri-apps/tauri#3701.
- **Frontend bundle must exist before any Rust build in CI:** with the default `custom-protocol` feature, `tauri::generate_context!` reads `frontendDist` (`../dist`) at compile time via a proc macro that panics when the directory is missing. Always order workflow steps as `npm ci` → `npm run build` → `cargo clippy/test`.
- **Diagnose from the failed log before touching anything:** `gh run view <id> --log-failed` filtered for `error|##[error]` pinpoints the failing build script in seconds; guessing produces wasted CI round-trips of 3+ minutes each.
