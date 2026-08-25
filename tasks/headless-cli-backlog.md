# Headless Automation & CLI — Backlog (Phase 18)

**Companions:** `headless-cli-spec.md`
**Epic Goal:** Terminal-driven collection execution with CI-friendly reports and exit codes.

**MVP Scope:** Stories S1–S5

---

## Story Map

```text
CLI CORE
S1 Arg parser, help/version text and headless branch in main()
S2 Input loading (collection/env/globals JSON) with --var overrides
S3 Execution wiring over RunCollectionUseCase + stdout summary + exit codes

REPORTING
S4 JSON envelope + JUnit XML report writers (infrastructure/reporting) + unit tests

PIPELINE INTEGRATION & DOCS
S5 GitHub Actions example workflow + docs sync (README, progress, AGENTS)
```

---

## Stories Breakdown

| ID | Story Title | Priority | Target Modules / Components | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **S1** ✅ | Arg parser, help/version and headless branch in `main()` | Must | `src-tauri/src/presentation/cli.rs`, `src-tauri/src/main.rs` | Pure std parsing; no new dependencies; subcommand dispatch before Tauri builder |
| **S2** ✅ | Input loading with `--var` overrides | Must | `presentation/cli.rs` | serde_json from paths; errors map to exit code 2 |
| **S3** ✅ | Execution wiring + stdout summary + exit codes | Must | `presentation/cli.rs` | Reuse adapters wired in main; PASS/FAIL lines per test; policy 0/1/3 |
| **S4** ✅ | Report writers + tests | Must | `src-tauri/src/infrastructure/reporting/` | JSON envelope + JUnit XML; XML escaping covered by tests |
| **S5** ✅ | GitHub Actions template + docs sync | Should | `docs/examples/`, `README.md`, `docs/progress.md`, `AGENTS.md` | Runnable workflow snippet using the binary artifact |

---

## Definition of Done (Epic)

- [x] S1–S5 completed and verified.
- [x] `tyny-pulse run <collection> --report out.junit` produces valid JUnit XML (escaped, well-formed) — validated against a local mock server E2E run.
- [x] Exit codes match the spec table (0/1/2/3) — all three paths exercised E2E.
- [x] `cargo check` and `cargo clippy -- -D warnings` pass without warnings.
- [x] New unit tests green (`cargo test`, 51 passing); boundary grep still returns 0 matches.
- [x] `npm run build` unaffected and green.
- [x] Docs describe the CLI contract accurately (README + progress tracker).
