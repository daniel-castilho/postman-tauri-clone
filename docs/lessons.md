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

- **QuickJS C sources do NOT compile under MSVC:** `quick-js`/`libquickjs-sys` build QuickJS C code that uses GCC extensions (`__attribute__((packed))` etc.), which `cl.exe` rejects (`cutils.h: syntax error: identifier 'packed_u64'`). Release builds for Windows must use the **GNU host toolchain** (`stable-x86_64-pc-windows-gnu`) — switching only the target is not enough, because `tauri-build`'s embed-resource then emits `resource.lib` in COFF/MSVC format and MinGW's `ld` rejects it ("file format not recognized"). macOS release builds use the `universal-apple-darwin` target (covers both `aarch64` and `x86_64`). See `.github/workflows/release.yml`.
- **Memory Limits in Sandbox:** Limit memory allocation for QuickJS runtime instances to prevent runaway user scripts from consuming desktop system RAM.

---

## 4. Pending Lessons & Future Log

_(This section is updated as new debugging lessons and infrastructure findings emerge during development.)_

---

## 5. CI/CD Lessons (GitHub Actions)

- **Tauri v2 requires the WebKitGTK 4.1 stack on Ubuntu CI runners:** installing the Tauri v1-era `libwebkit2gtk-4.0-dev` makes `javascriptcore-rs-sys` / `soup3-sys` build scripts fail with "The file `<lib>.pc` needs to be installed and PKG_CONFIG_PATH..." errors. The correct set (Ubuntu >= 22.04) is `libwebkit2gtk-4.1-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `pkg-config`. Sources: tauri-apps/tauri-action#720, tauri-apps/tauri#3701.
- **Frontend bundle must exist before any Rust build in CI:** with the default `custom-protocol` feature, `tauri::generate_context!` reads `frontendDist` (`../dist`) at compile time via a proc macro that panics when the directory is missing. Always order workflow steps as `npm ci` → `npm run build` → `cargo clippy/test`.
- **Diagnose from the failed log before touching anything:** `gh run view <id> --log-failed` filtered for `error|##[error]` pinpoints the failing build script in seconds; guessing produces wasted CI round-trips of 3+ minutes each.
- **The `secrets` context is NOT allowed in step-level `if:` conditionals:** a workflow containing `if: ${{ secrets.X != '' }}` fails validation at startup — runs fail in ~0s with "This run likely failed because of a workflow file issue", and worse, **every trigger silently stops registering** (even `on: pull_request` never appears as a PR check). Bridge the flag through job-level `env`, where `secrets` IS allowed:
  ```yaml
  jobs:
    coverage:
      env:
        HAS_CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN != '' }}
      steps:
        - if: env.HAS_CODECOV_TOKEN == 'true'
          uses: codecov/codecov-action@v5
  ```
- **Validate workflow files locally with `actionlint` before pushing:** it reproduces GitHub's exact schema/context validation (e.g. the `secrets`-in-`if` error above) in one command; a 0-second failed Actions run almost always means a workflow file issue, not a test failure.
