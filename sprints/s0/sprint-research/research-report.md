# Sprint s0 — Research Report

**Date:** 2026-05-19
**Sprint goal:** Bootstrap an all-Rust + WASM GUI for oovra: pick a stack,
restructure the repo into a workspace that supports both CLI-only and
full-GUI installs, and stand up a minimal compiling app shell that links
the `oovra` library crate.

## 1. Project context

`oovra` (this repo, `C:\Users\charl\oovra`) is a Rust CLI + library for
composing and comparing agentic prompt elements (Markdown + TOML). After
the 2026-05-19 redesign, prompt content is authored as plain `.md` files
and ingested via `oovra create --label` (in place) or `--olib` (copy into
`./olib/`); already-Oovra inputs to `--olib` are copied verbatim, giving
olib-to-olib transfer as a side effect. The library crate exposes the
parser, library loader, and the four operators — Create, Compose,
Decompose, Compare — and is suitable for direct use by a GUI front-end
(no subprocess required).

The user's GUI vision (from the directing message):

- An app to **select files** more easily than CLI paths allow.
- A built-in **code editor** so users can hand-author oovra particles
  without leaving the app.
- A **file explorer** that lets you click on filesystem locations, with
  **autosearch for an `olib/` in a target directory or below** so users
  don't have to fish for paths.
- A **live "autocompose" tab**: select/deselect files from an olib,
  drag to reorder, see the Oovra header tags inline without opening
  files, and create new versions of a prompt.
- **All Rust, including the web (WASM) build.**
- The CLI remains the substrate; the GUI is a frontend for humans, the
  CLI is for humans-on-terminals **and agents**. Installable in CLI-only
  mode (headless agent boxes) or full-GUI mode.

This research-report.md is the foundation; subsequent sprints will plan
and build specific GUI features. Sprint s0's scope is **bootstrap only**.

## 2. Codebase review — what the GUI will sit on top of

Top-level structure of `oovra` today:

- `src/lib.rs` — module roots (`create`, `decompose`, `diff`, `element`,
  `error`, `header`, `library`, `migrate`, `render`) and re-exports
  (`PromptElement`, `PromptElementHeader`, `PromptElementKind`,
  `Library`, `OovraError`, `parse_file`, `parse_file_with`, `write`).
- `src/main.rs` — the CLI binary, `clap` subcommands.
- `src/{create, element, header, library, render, diff, decompose,
  migrate}.rs` — operator modules.
- `tests/end_to_end.rs` — integration tests (23 currently).
- `Cargo.toml` — already a `[workspace]` (with the root package as the
  sole member today; this is the lever we'll use in s0).

Key library APIs the GUI will lean on directly:

- `Library::load_with(path, opts)` — walk an `olib/` directory into a
  parsed in-memory library. Perfect for the file-explorer / olib-list
  panel.
- `header::is_kebab_case`, `header::slugify`,
  `element::{parse, parse_file, write, looks_like_oovra_file}` — what
  the editor needs to validate / save user-edited atoms.
- `render::{compose, render_text, ComposeRequest}` — what the
  autocompose tab needs to live-render a selected subset.
- `create::{label_in_place, label_into_olib, copy_oovra_into_olib}` —
  what the "save into olib" button calls.
- `diff::compare` — for the "new version of a prompt" feature.

This is a clean substrate; nothing in `oovra` is CLI-specific behind
the `main.rs` binary. The library half stands on its own.

## 3. GUI framework survey

The user's hard constraints are **all-Rust** and **WASM-capable from
one codebase**. That narrows the field considerably. The surveyed
options:

### egui / eframe

- Immediate-mode GUI, pure Rust, by Emil Ernerfeldt ([emilk/egui](https://github.com/emilk/egui)).
- `eframe` is the framework wrapper: native via `winit` + `wgpu` (or
  `glow`), web via WebGL/WebGPU. **One codebase, native + web.**
- Current versions (per the official template, fetched 2026-05-19):
  `egui = "0.34.1"`, `eframe = "0.34.1"`, edition `2024`,
  `rust-version = "1.92"`.
- Official starter [`emilk/eframe_template`](https://github.com/emilk/eframe_template/)
  is exactly the boilerplate we need: `src/main.rs` (native entry),
  `src/lib.rs` (app + WASM `start` shim), `index.html` + Trunk for the
  web build.
- WASM build pipeline: `rustup target add wasm32-unknown-unknown`,
  `cargo install --locked trunk`, then `trunk serve` (dev) or
  `trunk build --release` (production → static `dist/`).
- Add-ons that map directly to the user's vision:
  - **Drag-reorder a list:** [`egui_dnd`](https://crates.io/crates/egui_dnd) —
    purpose-built for "reorder items by dragging" in a single list,
    exactly the autocompose-canvas requirement.
  - **Code editor with syntax highlighting:**
    [`egui_code_editor`](https://crates.io/crates/egui_code_editor) —
    a `TextEdit` widget with numbered lines, syntax highlighting from
    keyword sets, and optional autocompletion. Markdown can be added
    as a custom keyword set; the existing crate covers most cases.
  - **Native file picker:** [`rfd`](https://crates.io/crates/rfd) — used
    in `eframe`'s own examples for native file dialogs. On WASM it
    falls back to the browser's File picker.
- Tradeoff: **does not look "native"** (it draws its own widgets). The
  user is building a domain-specific tool, not a system-style file
  manager, so a distinctive look is fine — arguably desirable.

### Dioxus

- React-like, JSX-style RSX macros ([DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus)).
  Supports web, desktop, mobile, TUI.
- On desktop, Dioxus renders via **WebView2 / WebKitGTK** (essentially
  "Diet Electron" — it ships a browser engine). That contradicts the
  "all-Rust including in WASM" intent: the native desktop binary
  depends on the OS's web view, not on a pure-Rust renderer.
  ([rust-bytes article](https://weeklyrust.substack.com/p/the-state-of-rust-gui-the-good-and),
  [boringcactus 2025 survey](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html)).
- Strong web story, but the web/desktop unification depends on a
  browser engine on both ends.

### Iced

- Elm-inspired, retained-mode, pure Rust ([iced.rs](https://lib.rs/gui)).
- Native is mature; WASM support exists but is less proven than
  eframe's. Plausible alternative, but eframe's WASM story has more
  miles on it for the kind of UI we're building.

### Leptos / Yew

- Web-first WASM frameworks; desktop only via a wrapper like Tauri,
  which contradicts the all-Rust-everywhere intent.

### Slint, Freya, Xilem

- Slint has its own markup language — not "all Rust" in spirit.
- Freya (Dioxus + Skia) gives true native rendering but is younger.
- Xilem is promising but pre-1.0 and changing rapidly.

## 4. Recommendation: egui + eframe

For oovra's GUI, **egui + eframe** is the right pick:

1. **All-Rust by construction** — no embedded browser engine, no DSL,
   no JS toolchain. Native via `wgpu`, web via WebGL/WebGPU.
2. **Native and web from one codebase** is its core design goal.
   `eframe_template` proves the pipeline in production today.
3. **Direct widget coverage for the planned features**: `egui_dnd` for
   the drag-reorder compose canvas, `egui_code_editor` for the
   embedded particle editor, `rfd` for the file picker. Nothing in
   the user's vision requires reinventing core widgets.
4. **Immediate-mode model** fits the live-autocompose tab cleanly —
   re-deriving the rendered prompt every frame from the selected
   subset of atoms is the natural pattern, no diff bookkeeping
   needed.
5. **Stable, well-maintained, broadly adopted.** egui 0.34.x is the
   current line; the project has a clear release cadence and a large
   user base.

Tradeoff accepted: non-native look. For a domain tool whose users
already work in a CLI-and-Markdown world, this is fine — arguably an
asset (the GUI is distinct, not pretending to be a system app).

## 5. Workspace structure — CLI-only vs full-GUI install

Per the user's directive, the structure must allow both:

- **CLI-only**: `cargo install --path .` from the repo root installs
  only the existing `oovra` binary. No eframe / wgpu / winit downloads.
  Ideal for headless agent boxes.
- **Full GUI**: install / build the GUI crate, which links the
  `oovra` library crate directly.

Cargo's workspace model gives us this for free:

```
oovra/
  Cargo.toml          [workspace] members = [".", "gui"]
  src/                <- oovra lib + CLI binary (unchanged)
  gui/
    Cargo.toml        package = "oovra-gui", bin = "oovra-gui"
    src/main.rs       <- native entry
    src/lib.rs        <- App + WASM start shim
    src/app.rs        <- OovraApp (eframe::App impl)
    index.html        <- Trunk entry for the web build
  sprints/s0/...
  agent-tasks/...
```

`cargo install --path .` installs only the root package (the CLI).
`cargo install --path gui` installs the GUI binary. The GUI crate
depends on the root crate via `oovra = { path = ".." }`, so the GUI
binary always pulls a coherent, version-locked copy of the library.

This satisfies the user's constraint **structurally** — the GUI is
opt-in. A future refinement (sprint s2+) could add Cargo
`[features]` to gate optional GUI-side dependencies (e.g. drag-and-
drop / code editor crates), but s0 doesn't need feature flags.

## 6. WASM considerations and the file-system gap

The browser sandbox limits what the WASM build can do:

- **No direct filesystem reads** — `std::fs` is unavailable on
  `wasm32-unknown-unknown`. The web build can't autodiscover an
  `olib/` on the user's disk the way the native build can.
- The web build will need either (a) the **File System Access API**
  (Chromium-based browsers, via `web-sys`), (b) `<input type="file"
  webkitdirectory>` for one-shot folder uploads, or (c) operate
  against an in-memory / IndexedDB-backed virtual olib.
- **`rfd`** abstracts native and web file dialogs, but it can't grant
  arbitrary filesystem access on the web.

**Decision for s0:** scope the bootstrap to the **native** build only.
The WASM target is set up to *compile* (so we know the dep tree is
WASM-clean) but feature work that depends on filesystem access will be
factored behind a small abstraction trait so later sprints can swap in
the web implementation without rewriting feature code. Concretely:

- An `OlibSource` trait (in `gui/src/olib_source.rs`, future) with
  `discover()`, `load(path)`, `write(path, bytes)`.
- A native impl using `std::fs` + `oovra::Library::load_with`.
- A web impl using `web-sys` File System Access (later sprint).

For sprint s0 we don't introduce the trait yet — premature. We just
make sure no `std::fs` calls leak into shared code paths.

## 7. Risks and open questions

1. **Edition mismatch.** `oovra` is edition 2021; `eframe_template`
   ships at edition 2024. Mixing editions across workspace members is
   supported by Cargo. We'll set the `gui` crate to edition 2024 to
   match the template (and the upstream lint configuration), and
   leave the root crate at 2021 to avoid unrelated churn.
2. **Build size on first compile.** `eframe`'s dep tree (wgpu, winit,
   glow, image, etc.) is large. First `cargo build -p oovra-gui` will
   download and compile several hundred crates and may take many
   minutes on this machine. This affects only the GUI install path;
   the CLI install path is unaffected.
3. **`egui_code_editor` vs hand-rolled Markdown editor.** The crate
   exists and is maintained, but its Markdown syntax set may need
   tuning. Decision deferred to the sprint that builds the editor.
4. **CI.** The methodology calls for GitHub CI/CD verification in the
   Test Phase. We'll inspect whether CI exists in this repo during
   the Build Phase; pushing to trigger it is gated on the user
   authorizing a push. For s0 we verify locally and document the CI
   step as deferred until then.
5. **Plan-mode wrinkle.** The methodology says "engage plan mode and
   use maximum effort" in the Plan Phase. With "one sprint then check
   in" autonomy, the plan documents themselves are the artifact; the
   sprint does not stop mid-flight for ExitPlanMode approval.

## 8. References

- [emilk/egui — egui crate](https://github.com/emilk/egui)
- [eframe docs.rs](https://docs.rs/eframe/latest/eframe/)
- [emilk/eframe_template](https://github.com/emilk/eframe_template/)
- [eframe_template Cargo.toml](https://github.com/emilk/eframe_template/blob/main/Cargo.toml)
  (versions, edition, dependencies)
- [`egui_dnd` on crates.io](https://crates.io/crates/egui_dnd) — drag
  and drop sorting
- [`egui_code_editor` on crates.io](https://crates.io/crates/egui_code_editor) —
  code editor widget with numbered lines and syntax highlighting
- [`rfd` on crates.io](https://crates.io/crates/rfd) — native file dialogs
- [The State of Rust GUI — Rust Bytes](https://weeklyrust.substack.com/p/the-state-of-rust-gui-the-good-and)
- [2025 Survey of Rust GUI Libraries — boringcactus](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html)
- [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus) — for the
  Dioxus comparison
- [egui drag-and-drop discussion #3869](https://github.com/emilk/egui/discussions/3869)
- [egui native file dialog discussion #987](https://github.com/emilk/egui/discussions/987)

No external artifact files (PDFs, screenshots) were saved this sprint;
all evidence lives in the linked URLs above.
