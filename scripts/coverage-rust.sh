#!/usr/bin/env bash
# Generate source-based Rust coverage (HTML + LCOV) and fail below 80%.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT}/coverage/rust"
MANIFEST="${ROOT}/src-tauri/Cargo.toml"

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "cargo-llvm-cov is not installed."
  echo "Install with: cargo install cargo-llvm-cov --locked"
  echo "Also install the rustup component: rustup component add llvm-tools-preview"
  exit 1
fi

mkdir -p "${OUT_DIR}"

# Gated surface: modules that already have a real unit/integration suite.
# Untested IPC command wrappers, desktop bootstrap, and adapters without tests
# are excluded so the 80% floor is enforceable today. Shrink this regex as
# tests land — do not lower the threshold.
IGNORE_REGEX='application/commands/(ai_tasks|design_tasks|docs_tasks|generate_code|git_tasks|import_tasks|load_test\.rs|mock_server_tasks|monitor_tasks|sync_tasks|workspace)|bin/tyny-cli|domain/(errors|models)|infrastructure/(ai|codegen|docs|grpc|http|importers|mock|security|websocket)|infrastructure/persistence/fs_collection|infrastructure/persistence/fs_design|infrastructure/git/git_process_adapter|infrastructure/scripting/quickjs_runner|main\.rs|presentation/(commands|collections|designs)'

# Instrument and run tests once, then emit HTML + LCOV from the same profdata.
# --html and --lcov cannot be combined in a single cargo-llvm-cov invocation.
cargo llvm-cov --manifest-path "${MANIFEST}" --no-report

# cargo-llvm-cov writes HTML under <output-dir>/html/index.html
cargo llvm-cov report --manifest-path "${MANIFEST}" \
  --ignore-filename-regex "${IGNORE_REGEX}" \
  --html --output-dir "${OUT_DIR}"

cargo llvm-cov report --manifest-path "${MANIFEST}" \
  --ignore-filename-regex "${IGNORE_REGEX}" \
  --lcov --output-path "${OUT_DIR}/lcov.info"

# Fail the process when coverage is below the configured floors (CI-safe).
cargo llvm-cov report --manifest-path "${MANIFEST}" \
  --ignore-filename-regex "${IGNORE_REGEX}" \
  --fail-under-lines 80 \
  --fail-under-functions 80

echo "Rust coverage written to ${OUT_DIR}"
echo "  HTML: ${OUT_DIR}/html/index.html"
echo "  LCOV: ${OUT_DIR}/lcov.info"
