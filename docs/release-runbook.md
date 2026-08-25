# Tyny Pulse — Operational Release Runbook

How to build, sign, package, and release **Tyny Pulse** across desktop operating systems (Windows, macOS, Linux).

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 0. Release Artifacts & Targets

| Target OS | Installer Format | Architecture | Output Path |
| :--- | :--- | :--- | :--- |
| **Windows** | `.exe` (NSIS), `.msi` | `x86_64-pc-windows-msvc` | `src-tauri/target/release/bundle/nsis/` |
| **macOS** | `.dmg`, `.app` | `aarch64-apple-darwin` / `x86_64-apple-darwin` | `src-tauri/target/release/bundle/dmg/` |
| **Linux** | `.AppImage`, `.deb` | `x86_64-unknown-linux-gnu` | `src-tauri/target/release/bundle/appimage/` |

---

## 1. Local Production Build

To compile native production packages locally:

```bash
# 1. Install frontend dependencies
npm install

# 2. Execute Tauri production build
npm run tauri build
```

The compiled binaries will be placed under `src-tauri/target/release/bundle/`.

---

## 2. Automated Release Pipeline (GitHub Actions)

Releases are fully automated via GitHub Actions (`.github/workflows/release.yml`).

### Step-by-Step Release Flow:

1. **Verify Local Quality Gate:**
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   npm run build
   ```

2. **Update Version Numbers:**
   Ensure consistent versioning across:
   - `package.json` (`"version": "1.0.0"`)
   - `src-tauri/tauri.conf.json` (`"version": "1.0.0"`)
   - `src-tauri/Cargo.toml` (`version = "1.0.0"`)

3. **Commit & Tag Release:**
   ```bash
   git add .
   git commit -m "chore(release): prepare v1.0.0"
   git tag -a app-v1.0.0 -m "Tyny Pulse v1.0.0"
   git push origin main --tags
   ```

4. **CI Build & Code Signing:**
   The GitHub Actions workflow triggers automatically on `app-v*` tags:
   - Compiles native binaries for Windows, macOS, and Linux concurrently.
   - Code-signs Windows installers using Microsoft Authenticode.
   - Code-signs macOS binaries using Apple Developer ID and executes Notarization (`xcrun notarytool`).
   - Generates the `latest.json` manifest for the Tauri Auto-Updater service hosted on `tyny.ca`.
   - Creates a draft GitHub Release with attached installer assets.

---

## 3. Auto-Updater Service (`tyny.ca`)

Tyny Pulse includes built-in auto-update capabilities configured in `src-tauri/tauri.conf.json`:

```json
"plugins": {
  "updater": {
    "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IGF1dG91cGRhdGUgcHVibGljIGtleQ...",
    "endpoints": [
      "https://releases.tyny.ca/update/{{target}}/{{current_version}}"
    ]
  }
}
```

The updater endpoint on `releases.tyny.ca` reads the `latest.json` release manifest and provides signed update payloads directly to desktop clients.

---

## 4. Post-Release Verification Smoke Test

Before marking a release public:

1. Download generated installers on clean test VMs (Windows 11, macOS Sequoia, Ubuntu 24.04).
2. Install and launch **Tyny Pulse**.
3. Verify basic HTTP request execution (`GET https://api.tyny.ca/health`).
4. Verify local vault encryption / decryption.
5. Verify SpecHub OpenAPI linter loading.
