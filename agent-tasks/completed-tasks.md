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

---

## Sprint s2 — Embedded oovra-particle editor + inspect CLI

### CLI

- **C2.1–C2.3** (2026-05-20) — Added `Command::Inspect(InspectArgs)`
  + `run_inspect` to `src/main.rs`. Subcommand:
  `oovra inspect <file> [--format human|json]`. Human format shows
  header field-by-field (id / name / kind / version / meta plus
  any compound-only fields) and a body line/char count. JSON
  flattens the header and adds body summary fields. CLI reinstalled
  via `cargo install --path .`.

### GUI

- **G2.1** (2026-05-20) — Added `gui/src/editor.rs` with `Editor`
  struct, `OpenResult` enum, and `Editor::open` / `Editor::save` /
  `Editor::reload`. Three unit tests in the same file (round-trip
  edit, compound rejection, invalid-semver rejection).
- **G2.2** (2026-05-20) — Extended `OovraApp` state: added
  `atom_index`, `selected_atom`, `editor`, `compound_msg`, with
  `#[serde(skip)]` on all the heavy bits.
- **G2.3–G2.6** (2026-05-20) — Rewrote `app.rs::ui` for the
  3-column layout: top toolbar, two stacked left SidePanels
  (olibs / atoms), CentralPanel (editor view), bottom collapsing
  s0 probe panel. Selecting an atom calls `Editor::open`; the
  editor renders id (read-only), name / version / meta single-line
  TextEdits, body multi-line monospaced TextEdit, plus Save / Reload
  buttons with the dirty-flag-driven `Save *` label.
- **G2.7** (2026-05-20) — Removed `version`/`meta` from `AtomEntry`
  after a clippy dead_code warning surfaced the over-fit; the
  atom list now only needs `id`, `kind`, `path`. Atom details
  reach the user via the editor panel instead.

### Test Phase

- **T2.1** (2026-05-20) — `cargo test -p oovra` 64 PASS (unchanged
  from s1; s2 added no lib code).
- **T2.2** (2026-05-20) — `cargo test -p oovra-gui` **6 PASS**
  (s1's 3 + s2's 3 editor tests).
- **T2.3** (2026-05-20) — `cargo build --target
  wasm32-unknown-unknown -p oovra-gui` PASS.
- **T2.4** (2026-05-20) — `oovra inspect` CLI smoke against the
  demo tree atoms — human, json, and missing-file error path all
  behave correctly.
- **T2.5** (2026-05-20) — `cargo run -p oovra-gui` background;
  window up PID 61476, title `oovra-gui`. s2 layout active.
- **Sprint s2 close** (2026-05-20) — `unit-tests.md`,
  `integration-tests.md`, `e2e-tests.md`, `test-report.md`
  authored under `sprints/s2/sprint-tests/`. Acceptance criteria
  all satisfied.

- **s2 polish** (2026-05-20) — panel headers renamed to canonical
  terminology after user feedback: "olibs" → "Oovra Library
  Directories (Olibs)", "atoms" → "Library Components", central
  panel leads with "Component Editor". Default window 720×480 →
  960×600 so three columns breathe. No behavior changes.

---

## Sprint s3 — Live autocompose canvas

### GUI

- **G3.1** (2026-05-20) — Added `egui_dnd = "0.15"` to
  `gui/Cargo.toml`. egui_dnd 0.15.0 declares `egui ^0.34.0` so it
  composes cleanly with our existing 0.34.x line.
- **G3.2** (2026-05-20) — Created `gui/src/canvas.rs` with
  `CanvasState { order, output_id, status }` and methods
  `toggle / contains / live_preview / save_as_compound` over the
  existing `oovra::render` surface. Four unit tests in the same
  file: toggle idempotency, live preview equals direct
  `render_text`, save-as-compound round-trip, and missing-id
  surfaces `ElementNotFound`.
- **G3.3** (2026-05-20) — Extended `OovraApp` with `canvas:
  CanvasState`, a `CentralView { Editor, Canvas }` enum, and
  `view: CentralView`. All `#[serde(skip)]` so the canvas is a
  workbench, not a project document.
- **G3.4** (2026-05-20) — Library Components list gained a
  per-row checkbox: click on checkbox toggles canvas inclusion,
  click on the rest of the row opens the editor (independent
  click targets).
- **G3.5–G3.6** (2026-05-20) — Central panel gained a tab row
  (Editor / Canvas) under the "Component Editor" heading. The
  Canvas view renders a drag-reorderable list via
  `egui_dnd::dnd(ui, "...").show_vec(&mut canvas.order, ...)`,
  a live `render_text` preview in a read-only monospace
  multi-line TextEdit (re-rendered every frame), and a
  save-as-compound form with an output-id field + button.
- **G3.7** (2026-05-20) — `handle_canvas_save` reloads the
  active olib after a successful save so the new compound
  appears in Library Components without a manual reload.

### Test Phase

- **T3.1** (2026-05-20) — `cargo test -p oovra` 64 PASS (no lib
  regressions despite the canvas integration).
- **T3.2** (2026-05-20) — `cargo test -p oovra-gui` **10 PASS**
  (s2's 6 + s3's 4 new canvas tests).
- **T3.3** (2026-05-20) — `cargo build --target
  wasm32-unknown-unknown -p oovra-gui` PASS in 12.12s. egui_dnd
  + its transitive `egui_animation` both compile cleanly for
  wasm32.
- **T3.4** (2026-05-20) — `cargo run -p oovra-gui` background;
  s3 layout active. Window confirmed at PID 64700.
- **clippy fix** (2026-05-20) — added
  `#![allow(clippy::result_large_err)]` to `gui/src/lib.rs` to
  mirror the root crate's existing suppression for the same
  `OovraError`-is-large reason.

### Cross-platform check (mid-sprint user feedback)

- **WSL bootstrap** (2026-05-20) — Installed stable Rust
  (`cargo 1.95.0 / rustc 1.95.0`) into the WSL Ubuntu image via
  rustup with `--profile minimal`.
- **Linux build** (2026-05-20) — `cargo build -p oovra` Linux-side
  via WSL with `CARGO_TARGET_DIR=/tmp/oovra-linux-target` PASS in
  13.70s.
- **Linux test** (2026-05-20) — Full `cargo test -p oovra` on
  Ubuntu via the same WSL invocation: **24 integration tests
  PASS**, lib + main unit tests PASS, exit 0.
- **Project policy** (2026-05-20) — `oovra/CLAUDE.md` updated
  with a "Cross-platform requirement" section codifying the
  Ubuntu + WSL workflow. Memory entry
  `feedback-oovra-cross-platform-and-wsl` added.

### Sprint s3 close

- Sprint-tests docs authored. Acceptance criteria met. Window
  left up for visual review. Roadmap next: s4 — diff/versioning
  view.

- **s3 polish** (2026-05-20) — live preview rewritten from a
  bordered monospace `TextEdit` to a selectable wrapped `Label`
  per user feedback — same font as the rest of the UI.

---

## Sprint s4 — Diff / versioning view

### Library

- **L4.1** (2026-05-20) — Added `BumpKind { Patch, Minor, Major }`
  enum and `bump_version(v: &str, kind: BumpKind)
  -> Result<String, String>` to `src/header.rs`. Uses
  `semver::Version`; strips pre-release / build-metadata on bump.
- **L4.3** (2026-05-20) — Five `#[test]`s in `header::tests`
  (patch / minor / major / pre+build stripped / garbage rejected).

### CLI

- **C4.1–C4.3** (2026-05-20) — Added `Command::BumpVersion` +
  `run_bump_version` to `src/main.rs`. Subcommand:
  `oovra bump-version <FILE> [--bump patch|minor|major]`.
  Prints `Bumped <id>: <old> -> <new> at <path>`.
- **C4.4** (2026-05-20) — Added
  `bump_version_round_trips_an_atom` integration test in
  `tests/end_to_end.rs`: label_into_olib → parse → bump → write
  → re-parse → assert new version.

### GUI

- **G4.1** (2026-05-20) — `CentralView::Compare` variant + a
  third selectable_value in the tab row.
- **G4.2** (2026-05-20) — Added `gui/src/canvas.rs`'s sibling
  `gui/src/compare.rs` with `CompareState { a, b, report }` and
  methods `set_a / set_b / recompute`. Four unit tests covering
  empty-pick / content / mixed-kind / same-id paths.
- **G4.3** (2026-05-20) — Implemented `render_compare` in
  `app.rs`: two ComboBox pickers populated from the loaded olib;
  router for None / Err / Ok report states. New free function
  `render_diff_report` renders both DiffReport variants —
  Content with a 3-column field-changes Grid and a colored
  body unified diff; Structural with added / removed /
  version_changed / moved sections.
- **G4.4** (2026-05-20) — Added a `Bump patch` button in
  `render_editor`'s button row. Calls `bump_version` and updates
  `editor.version` in memory; dirty flag flips so the next Save
  persists.
- **Clear-on-olib-change** (2026-05-20) — `clear_olib_selection`
  resets `compare` too so stale picks don't survive.

### Test Phase

- **T4.1** (2026-05-20) — `cargo test -p oovra` **70 PASS** (41
  lib unit + 4 main unit + 25 integration; was 36 / 4 / 24).
- **T4.2** (2026-05-20) — `cargo test -p oovra-gui` **14 PASS**
  (10 prior + 4 new compare).
- **T4.3** (2026-05-20) — `cargo build --target
  wasm32-unknown-unknown -p oovra-gui` PASS, 13.25s.
- **T4.4** (2026-05-20) — `oovra bump-version` CLI smoke against
  the mock library — bumped tone-direct 1.0.0 → 1.0.1, verified
  with `oovra inspect`.
- **T4.5** (2026-05-20) — `oovra-gui` window up PID 6164 with
  the new Compare tab + Bump patch button.
- **T4.6** (2026-05-20) — WSL Ubuntu `cargo test -p oovra` PASS,
  25 integration tests + lib + main, exit 0. Cross-platform
  invariant from CLAUDE.md still holds.

### Sprint s4 close

- Sprint-tests docs authored at `sprints/s4/sprint-tests/`.
  Acceptance criteria all satisfied. Window left up for visual
  review. Roadmap next: **s5 — WASM filesystem shim + Trunk
  pipeline** (higher-leverage) or s6 — egui Panel-alias
  migration (lower-effort cleanup).

---

## Sprint s5 — Filename-suffix versioning + Save/Save-As-New-Version + tree view

**Note:** s5 was redirected from the original roadmap pickup (WASM
filesystem shim) to versioning work after user feedback. The shim
is still on the roadmap.

### Library

- **L5.1** (2026-05-20) — Added `header::parse_filename_version`
  that detects `<canonical>-v<X>-<Y>-<Z>` at the end of a stem
  (last-occurrence, all-ASCII-digit groups). Seven unit tests.
- **L5.2** (2026-05-20) — Added `header::compose_versioned_filename`
  (the inverse). Three unit tests (round-trip, strips pre+build,
  rejects garbage).
- **L5.3** (2026-05-20) — Added `BumpKind::default() = Patch` so
  the GUI can store a `BumpKind` field with `#[serde(skip)]`.

### CLI

- **C5.1–C5.2** (2026-05-20) — Added `Command::ForkVersion` +
  `run_fork_version` to `src/main.rs`. Subcommand:
  `oovra fork-version <file> [--bump patch|minor|major]`. Writes
  a sibling at `<dir>/<canonical>-v<dashed-new>.md` with header
  fields set: `id = full stem`, `name = canonical`, `version =
  bumped`. Original file untouched.
- **C5.3** (2026-05-20) — Added integration test
  `fork_version_creates_versioned_sibling` in
  `tests/end_to_end.rs`.

### GUI

- **G5.1** (2026-05-20) — Editor field relabels: "id" → "Filesystem
  Name" (read-only TextEdit), "name" → "Component-ID" (editable),
  "version" + "meta" unchanged.
- **G5.2** (2026-05-20) — Editor::open auto-parses the filename
  suffix; if a version is present in the suffix, it overrides the
  in-memory editor version. Component-ID auto-fills with the
  parsed canonical id when `header.name == header.id` (uncustomized
  default).
- **G5.3** (2026-05-20) — Editor button row redesigned:
  **Save** (sets `save_confirm_pending`), **Save As New Version**
  (calls `save_as_new_version_now`), bump-kind ComboBox (Patch /
  Minor / Major), Reload, Bump (no-fork). The Save button shows
  a confirm Window with the consequence spelled out.
- **G5.4** (2026-05-20) — Library Components replaced with a
  recursive tree (`render_component_tree` + `render_tree_node`).
  Atoms render as leaf rows with checkbox + selectable label.
  Compounds render as `CollapsingHeader` containing recursive
  node renders. Defensive depth cap at 16.
- **G5.5** (2026-05-20) — Expand all / Collapse all buttons in the
  Library Components header; drive a `pending_open: Option<bool>`
  flag consumed once per frame by every CollapsingHeader.
- **G5.6** (2026-05-20) — `save_as_new_version_now`: parses the
  current editor's filesystem name, bumps the version, composes
  the new sibling filename, writes a fresh PromptElement, then
  reloads the active olib so the sibling appears in the tree.
- **G5.7** (2026-05-20) — `render_save_confirm` Window in `ui()`
  — modal-ish "Are you sure?" with consequences spelled out;
  Yes triggers `editor.save()`, Cancel clears the pending flag.

### Test Phase

- **T5.1** (2026-05-20) — `cargo test -p oovra` **81 PASS** (51
  lib unit + 4 main unit + 26 integ; was 70 in s4).
- **T5.2** (2026-05-20) — `cargo test -p oovra-gui` **15 PASS**
  (3 app + 4 editor + 4 canvas + 4 compare).
- **T5.3** (2026-05-20) — `cargo build --target
  wasm32-unknown-unknown -p oovra-gui` PASS, 11.47s.
- **T5.4** (2026-05-20) — `oovra fork-version` CLI smoke against
  the mock library: forked `citation-discipline` v1.0.0 →
  `citation-discipline-v1-0-1` v1.0.1, verified via `oovra
  inspect`.
- **T5.5** (2026-05-20) — Composed a higher-order compound
  `meta-agent` (body_level 2) in the mock library that includes
  the existing `coding-agent-prompt` compound — gives the GUI
  tree two real levels of recursion to demonstrate.
- **T5.6** (2026-05-20) — `oovra-gui` window up at PID 61696
  with the new tree, Save / Save As New Version flow, and the
  field relabels.
- **T5.7** (2026-05-20) — WSL Ubuntu `cargo test -p oovra` PASS,
  identical 51+4+26 = 81 tests. Cross-platform invariant intact.

### Sprint s5 close

- Sprint-tests docs authored at `sprints/s5/sprint-tests/`.
  Acceptance criteria all satisfied. Window left up. Roadmap
  next: **s6 — egui Panel-alias migration** OR return to **WASM
  filesystem shim** (originally roadmap s5).

- **s5 fix: correlated list items** (2026-05-20) — the cascade
  polish only deduped at the top level; within-tree diamond dupes
  (an atom in two compounds of the same DAG) still rendered twice
  with synchronized checkboxes. Added Library::component_tree()
  (deduped forest, seen-set, depth-first pre-order from roots) +
  ComponentNode; gui render_tree_node → render_component_node
  driving off the deduped tree. Diagnostic + fix + regression
  unit tests in library.rs. NOTE: the prior build hadn't actually
  deployed (a running .exe held a file lock so `cargo build
  --quiet` silently failed); confirmed fresh-build timestamp and
  relaunched. User confirmed the clean schema.

---

## Sprint s6 — egui Panel-alias migration

- **Research** (2026-05-20) — confirmed against docs.rs/egui/0.34.2:
  TopBottomPanel/SidePanel are deprecated aliases for the unified
  `Panel`; `default_width`/`default_height` deprecated for the
  orientation-aware `default_size`.
- **Build** (2026-05-20) — `gui/src/app.rs`: migrated the toolbar,
  footer, olibs, and components panels to `egui::Panel::{top,
  bottom,left}`; `default_width` → `default_size`; removed the
  `#[allow(deprecated)]` on `App::ui`.
- **Test** (2026-05-20) — clippy clean (deprecation warnings gone);
  workspace 104 PASS unchanged; wasm32 build PASS (11.24s); WSL
  Ubuntu 88 PASS parity; GUI relaunched (PID 62244) — three-column
  layout visually identical.
- **Sprint s6 close** — docs at `sprints/s6/`. Roadmap next:
  syntax-highlighted editor + markdown preview, WASM filesystem
  shim, compound recipe editing, or cross-olib compare.

---

## Sprint s7 — Inline header tags in the component tree

- **Build** (2026-05-20) — `gui/src/app.rs::render_component_node`
  reads `version` + `meta` per node and renders them inline via a
  new `tag_labels(ui, version, meta)` free fn (dimmed `v<version>`
  + truncated `— <meta>`). Delivers the original-vision item "see
  the tags from the Oovra header without opening." Display-only.
- **Test** (2026-05-20) — clippy clean; workspace 104 PASS
  unchanged; wasm32 PASS; WSL Ubuntu 88 PASS parity; GUI relaunched
  (PID 66464) showing tags inline.
- **Sprint s7 close** — docs at `sprints/s7/`.
