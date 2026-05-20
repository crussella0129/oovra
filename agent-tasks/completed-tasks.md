# completed-tasks.md

Log of all completed agentic coding work, appended in completion order
as tasks are dispatched from `agent-tasks.md`. Persists across sprints.

---

## Sprint s0 — Project Bootstrap

### Build Phase

- **A.1** (2026-05-19) — Edited root `Cargo.toml`: `[workspace]
  members = [".", "gui"]`. CLI-only install path preserved (`cargo
  install --path .` still installs only the root package).
- **B.1** (2026-05-19) — Created `gui/Cargo.toml` (package
  `oovra-gui`, edition 2024, rust-version 1.92, deps `egui = "0.34"`,
  `eframe = "0.34"` + persistence, `oovra = { path = ".." }`, and
  per-target wasm32 / non-wasm32 deps).
- **B.4** (2026-05-19) — Created `gui/src/main.rs`: native main
  (`eframe::run_native`, 720x480 window) and WASM main
  (`wasm_bindgen_futures::spawn_local` driving `eframe::WebRunner` on
  `#the_canvas_id`), cfg-gated.
- **B.2** (2026-05-19) — Created `gui/src/app.rs`: `OovraApp` (serde
  Deserialize/Serialize, eframe persistence wired) with a live
  `is_kebab_case` / `slugify` probe driven by user input — proves
  the gui crate calls real `oovra::header` functions at runtime. Also
  added `oovra::VERSION` constant in `src/lib.rs` so the gui can show
  the linked lib version.
- **B.3** (2026-05-19) — Created `gui/src/lib.rs`: `mod app; pub use
  app::OovraApp;` plus the standard lint warns.
- **B.5** (2026-05-19) — Created `gui/index.html`: Trunk web entry
  with `<link data-trunk rel="rust" data-bin="oovra-gui"
  data-wasm-opt="2"/>`, `#the_canvas_id` canvas, and a loading
  indicator the WASM `main` removes on successful start.
- **C.1** (2026-05-19) — Updated root `README.md`: documents the
  workspace, separates the CLI-only install (`cargo install --path
  .`) from the full-GUI install (`cargo install --path gui`),
  mentions Trunk for the WASM build.

### Test Phase

- **D.1** (2026-05-19) — `cargo build -p oovra` exit 0 (CLI unaffected
  by the workspace stanza).
- **D.2** (2026-05-19) — `cargo test -p oovra` 57/57 PASS (30 + 4 + 23).
- **Test Phase fix** (2026-05-19) — eframe 0.34.2 trait drift caught
  during the first `oovra-gui` build: the `App` trait required `ui`
  (not `update`). Patched `gui/src/app.rs` to implement
  `App::ui(&mut self, ui, frame)` and dropped the `CentralPanel`
  wrapper. Root cause + lesson recorded in `test-report.md` §3.
- **D.3** (2026-05-19) — `cargo test -p oovra-gui` (debug) finished
  in 44.13s. Smoke test `default_app_is_constructible_and_oovra_lib_is_reachable`
  PASS — proves the gui crate links the workspace's `oovra` lib and
  exercises its API at test time.
- **D.4** (2026-05-19) — `cargo build --target wasm32-unknown-unknown
  -p oovra-gui` finished in 1m 06s, exit 0. WASM dep tree resolves;
  shared code paths are free of `std::fs` / `std::net` leaks.
- **D.5** (2026-05-19) — `cargo run -p oovra-gui` launched the
  native window. PID 58088, window title "oovra-gui". User-confirmed
  visual content: bootstrap title, `oovra v0.2.0` link line, and the
  live `is_kebab_case` / `slugify` probe rendering on the default
  `"My Draft"` value as `false` / `Some("my-draft")`. E-1 PASS.
- **Sprint s0 close** (2026-05-19) — `unit-tests.md`,
  `integration-tests.md`, `e2e-tests.md`, and `test-report.md`
  authored in `sprints/s0/sprint-tests/`. Acceptance criteria all
  satisfied. The s0 + create-redesign work landed as two commits
  on branch `feat/create-redesign-and-gui-bootstrap`.

---

## Sprint s1 — Olib autodiscovery + file explorer panel

### Library

- **L.1** (2026-05-19) — Added `src/discovery.rs` with
  `DiscoveredOlib { path, md_count }` and `discover(root, max_depth)`.
  Walks via `walkdir`, filters dirs named `olib` (case-insensitive),
  counts `.md` files non-recursively, sorts by path. Errors
  mid-walk are stderr'd and skipped.
- **L.2** (2026-05-19) — Wired into `src/lib.rs`; re-exported
  `discover` and `DiscoveredOlib` at the crate root.
- **L.3** (2026-05-19) — Six unit tests in `discovery.rs::tests`
  covering shallow / deep / max_depth / non-olib / missing-root /
  deterministic ordering.

### CLI

- **C.1–C.3** (2026-05-19) — Added `Command::Discover(DiscoverArgs)`
  and `run_discover()` in `src/main.rs`. Subcommand:
  `oovra discover <root> [--max-depth N] [--format human|json]`.
  Human format prints a colored summary; JSON format prints a
  single-line `serde_json` array. CLI re-installed via
  `cargo install --path .`.
- **C.4** (2026-05-19) — Added `discover_finds_two_nested_olibs`
  integration test in `tests/end_to_end.rs` — builds a temp tree
  with two olibs and a decoy, asserts paths + md counts.

### GUI

- **G.1** (2026-05-19) — Added `rfd = "0.15"` to `gui/Cargo.toml`
  behind `cfg(not(target_arch = "wasm32"))` so the wasm32 build
  doesn't pull it in.
- **G.2 / G.3 / G.4** (2026-05-19) — Rewrote `gui/src/app.rs` for
  the s1 layout: top toolbar with `Open folder…` (native: rfd;
  wasm32: deferred-feature stub), left sidebar listing discovered
  olibs, central panel showing the selected olib's atoms with id /
  kind / version / meta. The s0 kebab/slugify probe is preserved
  in a collapsing section for sprint-over-sprint visual diff.
  Heavy state fields `#[serde(skip)]`; persisted only
  `kebab_probe` and `roots`. Added a `short_path` helper +
  two unit tests; expanded the existing smoke test to assert the
  new state fields initialize empty.

### Test Phase

- **Issue 1 fix** (2026-05-20) — egui 0.34.2 deprecated
  `TopBottomPanel`/`SidePanel` aliases and `default_width`.
  Silenced with `#[allow(deprecated)]` on the `App::ui` impl and
  a TODO note for a follow-up migration sprint.
- **Issue 2 fix** (2026-05-20) — `open_folder` flagged dead on
  wasm32 (its only caller is the native rfd branch). Added
  `#[cfg_attr(target_arch = "wasm32", allow(dead_code))]`.
- **T.1** (2026-05-19) — `cargo test -p oovra` 36 + 4 + 24 = **64
  tests PASS** (57 prior + 6 lib unit + 1 integration).
- **T.2** (2026-05-20) — `cargo test -p oovra-gui` 3 tests PASS in
  15.56s.
- **T.3** (2026-05-20) — `cargo build --target
  wasm32-unknown-unknown -p oovra-gui` PASS, 9.94s.
- **T.4** (2026-05-19/20) — `oovra discover` CLI smoke (human +
  json) PASS against the s1 demo tree at `C:\Users\charl\oovra-demo`.
- **T.5** (2026-05-20) — `cargo run -p oovra-gui` background;
  window up PID 57812, title `oovra-gui`. Layout matches the spec;
  awaiting the user's visual review whenever they return.
- **Sprint s1 close** (2026-05-20) — `unit-tests.md`,
  `integration-tests.md`, `e2e-tests.md`, `test-report.md`
  authored. Acceptance criteria all satisfied.
