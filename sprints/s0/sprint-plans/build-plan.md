# Sprint s0 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-19)

Sprint goal: restructure `oovra` into a Cargo workspace with a new
`gui` crate (`oovra-gui`); stand up a minimal compiling eframe app
shell that links the `oovra` library; keep `cargo install --path .`
as a CLI-only install path; document and verify everything.

Source: see [`../sprint-research/research-report.md`](../sprint-research/research-report.md)
for the framework decision (egui + eframe 0.34) and rationale.

---

## 1. Full project schema

```
oovra (workspace root, this repo)
│
├── ROOT CRATE  — package "oovra"  (unchanged in s0 except the workspace
│   │                                stanza in Cargo.toml)
│   ├── library half (src/lib.rs + modules)
│   │     parser, header, element, library loader, operators
│   │     -> the surface the GUI crate links against
│   └── CLI half (src/main.rs)
│         create / compose / decompose / compare / migrate
│
└── GUI CRATE  — package "oovra-gui"  (NEW, added in s0)
    ├── App shell                              <-- s0 deliverable
    │     eframe::App impl, native + WASM
    ├── Olib source abstraction               (s2+)
    │     trait OlibSource { discover, load, write }
    │     - native impl via std::fs + oovra::Library
    │     - web impl via web-sys File System Access
    ├── Panels                                 (s1+)
    │     - File explorer (tree, olib autodiscovery)
    │     - Olib list (atoms + compounds)
    │     - Editor (markdown, oovra header, syntax-highlighted)
    │     - Autocompose canvas (select, drag-reorder, live render)
    ├── Diff / versioning view                 (s3+)
    │     built on oovra::diff::compare
    └── Persistence (eframe persistence)       (s1+)
          window state, last olib, panel layout
```

s0 builds *only* the App shell branch; everything else is later-sprint
scope. The schema is recorded here so later sprints have a stable
mental model to fit work into.

## 2. s0 sub-schema (elementary)

```
s0 = "workspace + gui crate + minimal compiling app shell"
│
├── A. Workspace restructure
│     A.1  Edit Cargo.toml (root): [workspace] members = [".", "gui"].
│     A.2  Verify CLI-only install path unchanged:
│            `cargo install --path .` builds only the oovra binary.
│
├── B. GUI crate scaffold
│     B.1  Create gui/Cargo.toml
│           - package: name "oovra-gui", edition 2024,
│             rust-version "1.92", license dual MIT/Apache-2.0
│           - lib  : crate-type ["cdylib", "rlib"]  (cdylib for wasm-bindgen)
│           - bin  : name "oovra-gui", path "src/main.rs"
│           - deps : oovra (path "..", default-features off — not yet
│                       relevant), egui 0.34, eframe 0.34 + persistence,
│                       log 0.4
│           - target-specific deps:
│              not(wasm32) -> env_logger 0.11
│              wasm32      -> wasm-bindgen 0.2, wasm-bindgen-futures 0.4,
│                              web-sys 0.3
│     B.2  Create gui/src/app.rs
│           - pub struct OovraApp { greeting: String, kebab_probe: String }
│           - Default impl: greeting from a CARGO_PKG_VERSION snippet,
│             kebab_probe seeded with a sample string.
│           - impl eframe::App for OovraApp { fn update(...) }
│             CentralPanel:
│               - h1 "oovra-gui — sprint s0 bootstrap"
│               - line showing oovra crate version (via the `oovra`
│                 lib re-export of CARGO_PKG_VERSION or a const).
│               - TextEdit on kebab_probe; live label showing
│                 `is_kebab_case(&kebab_probe)` and the slugified form
│                 — this PROVES the gui crate is linking the oovra lib
│                 and calling its real API.
│     B.3  Create gui/src/lib.rs
│           - mod app; pub use app::OovraApp;
│           - #[cfg(target_arch = "wasm32")] use wasm_bindgen::prelude::*;
│             #[wasm_bindgen]
│             pub async fn start(canvas_id: String) -> Result<(), JsValue>
│             that calls eframe::WebRunner.
│     B.4  Create gui/src/main.rs (native entry only)
│           - #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
│           - #[cfg(not(target_arch = "wasm32"))] fn main() -> eframe::Result
│             that initializes env_logger and calls eframe::run_native.
│     B.5  Create gui/index.html (Trunk entry)
│           - canvas element, link-rel for Trunk to insert the WASM
│             bundle; data-trunk script for the lib crate.
│           - matches the structure of eframe_template's index.html.
│     B.6  Create gui/Trunk.toml (optional, defaults usually fine)
│           - skip unless needed.
│
├── C. Documentation
│     C.1  Update root README:
│           - mention the workspace and the two install paths
│             ("CLI only" vs "GUI" with the exact `cargo install` cmds).
│           - point at `gui/` and Trunk for the web build.
│
├── D. Build verification (lives in the Test Phase, listed here so
│     dependencies are explicit)
│     D.1  cargo build -p oovra            -- CLI unchanged (fast).
│     D.2  cargo test  -p oovra            -- existing 57 tests still pass.
│     D.3  cargo build -p oovra-gui        -- native; slow first time
│                                              (downloads eframe tree).
│     D.4  rustup target add wasm32-unknown-unknown
│           cargo build --target wasm32-unknown-unknown -p oovra-gui
│                                          -- proves WASM dep tree resolves
│                                              (no std::fs in shared code).
│     D.5  cargo run -p oovra-gui          -- launches the native app for
│                                              visual verification (per
│                                              the user's "launch for
│                                              visual review" feedback).
│     D.6  cargo install --path .          -- CLI-only install path
│                                              unaffected (sanity check;
│                                              skip if D.1/D.2 pass — the
│                                              install just re-exercises
│                                              D.1).
│
└── E. Commit hygiene
      E.1  Two logical commits ready (not auto-pushed):
            (i) the `create` redesign already on disk from earlier today,
            (ii) the s0 bootstrap (workspace + gui + sprint dirs +
                  agent-tasks).
            Pushing is gated on the user's say-so.
```

## 3. Execution sequence (the elementary unit ordering)

The order below minimizes time-to-first-failure: the slow step (D.3)
sits after every cheap step has either succeeded or surfaced an error.

1. **A.1** — edit `Cargo.toml` to add workspace `members`.
2. **B.1** — write `gui/Cargo.toml`.
3. **B.4** — write `gui/src/main.rs` (small, lets `cargo metadata`
   see the bin target).
4. **B.2** — write `gui/src/app.rs` (the App struct & impl).
5. **B.3** — write `gui/src/lib.rs` (lib roots + WASM start).
6. **B.5** — write `gui/index.html`.
7. **C.1** — update root README.
8. **D.1** — `cargo build -p oovra` (fast sanity).
9. **D.2** — `cargo test -p oovra` (regression check on existing
   tests against the workspace-restructured Cargo.toml).
10. **D.3** — `cargo build -p oovra-gui` (slow; run in background
    if needed).
11. **D.4** — WASM target build (after D.3 succeeds; reuses much of
    the cached build graph).
12. **D.5** — `cargo run -p oovra-gui` in the background; verify a
    window opens. Per the user's saved feedback, this gives them a
    window for visual review rather than asking them to run it
    themselves.
13. **E.1** — list outstanding logical commits in the sprint report;
    do not commit/push without explicit authorization.

## 4. Important execution details

- **Edition mixing.** Root `oovra` package stays at edition `2021`;
  `oovra-gui` is edition `2024` to match the eframe template. Cargo
  supports per-member editions cleanly; no workspace-level edition is
  set in `Cargo.toml`.
- **CLI-only install path is sacred.** `cargo install --path .` must
  not download or compile eframe / wgpu / winit. Workspace members
  are NOT built by `cargo install --path .`; that command builds only
  the package at the given path. We verify this by running it after
  the restructure and confirming the install completes quickly with
  no eframe-stack downloads. (Sanity, not a test gate.)
- **`oovra` path-dep.** `gui/Cargo.toml` depends on the root crate
  via `oovra = { path = ".." }`. This pulls the library half (lib
  target) automatically — no `default-features = false` gymnastics
  needed because the root crate doesn't currently expose features.
  This will be revisited when/if the root crate splits binary-only
  deps behind a feature flag.
- **No new GUI deps yet.** `egui_dnd`, `egui_code_editor`, `rfd`, and
  the file-system-access shims are explicitly out of s0 scope. s0
  takes only `egui`, `eframe`, `log`, and the wasm32-side bindings.
  Resisting dep creep here keeps the first cargo build manageable.
- **WASM build verification, NOT WASM running.** We confirm `cargo
  build --target wasm32-unknown-unknown -p oovra-gui` succeeds; we do
  NOT install Trunk or serve the web build this sprint. That's a
  future sprint's concern; documenting the Trunk steps in the README
  is enough.
- **Window launching from a tool.** `cargo run -p oovra-gui` opens a
  blocking event loop. Run it `run_in_background = true` so it stays
  up across tool calls. The user can interact; we don't need to
  capture stdout.
- **No `.gitignore` changes** for `sprints/` or `agent-tasks/`. They
  are project artifacts the user may want to commit.

## 5. Acceptance criteria for s0

- `cargo build -p oovra-gui` succeeds on native.
- `cargo build --target wasm32-unknown-unknown -p oovra-gui` succeeds
  (no `std::fs` leaks into shared code paths).
- `cargo test -p oovra` still reports the pre-existing test count
  passing.
- `cargo run -p oovra-gui` opens a window that displays the bootstrap
  greeting AND a live `is_kebab_case` / `slugify` probe driven by
  user input — proving the gui crate links and uses the oovra lib's
  real API at runtime, not just at compile time.
- `cargo install --path .` still produces a CLI-only install with no
  eframe stack downloads.
- A populated `agent-tasks/` directory with the s0 build tasks moved
  to `completed-tasks.md`.
