# Sprint s1 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-19)

Sprint goal: ship **olib autodiscovery** in the library, expose it
via `oovra discover` on the CLI, then surface it in `oovra-gui` as a
file-explorer panel that calls the same library function.

Source: [`../sprint-research/research-report.md`](../sprint-research/research-report.md).

CLI-first rule from [`/CLAUDE.md`](../../../CLAUDE.md) is the load-bearing
ordering constraint: library → CLI → GUI.

## 1. Sub-schema (elementary)

```
s1 = "olib discovery: library function + CLI cmd + GUI panel"
│
├── L. Library (lands first)
│     L.1  Add `src/discovery.rs` with:
│            - pub struct DiscoveredOlib { path: PathBuf, md_count: usize }
│            - pub fn discover(root, max_depth) -> Result<Vec<DiscoveredOlib>>
│            - private helper to walk via `walkdir` (already a workspace dep)
│     L.2  Register the module in `src/lib.rs` and re-export
│            DiscoveredOlib + discover at crate root.
│     L.3  Unit tests for discover:
│            - shallow hit, deep hit, max_depth bound, non-olib dirs ignored,
│              non-existent root errors, plain `.md` file at root not counted.
│
├── C. CLI (lands second; depends on L)
│     C.1  Add `Command::Discover(DiscoverArgs)` to `src/main.rs`.
│     C.2  DiscoverArgs fields: root: PathBuf (positional),
│            max_depth: Option<usize> (--max-depth), format: String
│            (--format, default "human").
│     C.3  run_discover() function: call library::discover, print
│            either the human pretty form or a one-line JSON via
│            `serde_json::to_string`.
│     C.4  An integration test in `tests/end_to_end.rs` that builds
│            a small temp tree (two nested `olib/` dirs with .md
│            files) and exercises discover() at the library level.
│            (CLI invocation tests can come later if needed —
│            library coverage is the load-bearing layer.)
│
├── G. GUI (lands last; depends on L)
│     G.1  Add `rfd = "0.15"` to `gui/Cargo.toml`. Pin the major
│            version so first-build deps don't drift.
│     G.2  Extend `OovraApp` state: roots (Vec<PathBuf>), discovered
│            (Vec<DiscoveredOlib>), selected (Option<usize>), and the
│            optional `Library` of the selected olib.
│     G.3  Restructure `app.rs::ui` to:
│            - TopBottomPanel "toolbar": [Open folder…] button calls
│              `rfd::FileDialog::new().pick_folder()` (sync, native).
│              On success, runs `oovra::discovery::discover(root, None)`
│              and stores results in state.
│            - SidePanel "olibs" (left, resizable, default 220px):
│              list of `DiscoveredOlib` entries with a selectable
│              label per item; selecting one loads the olib via
│              `oovra::Library::load_with` and stashes it in state.
│            - CentralPanel: header + the selected olib's atoms,
│              each row showing id, meta. The s0 kebab/slugify probe
│              stays at the bottom for sprint-over-sprint visual diff.
│     G.4  Add `serde` skip for the heavy state fields (`Library`,
│            `Vec<DiscoveredOlib>`) — they shouldn't persist across
│            runs; only `roots` is worth persisting.
│
└── T. Tests + verification (driven by test-plan.md; see for detail)
      T.1  cargo test -p oovra                  -- 57 + new discover tests pass.
      T.2  cargo test -p oovra-gui              -- gui smoke still passes.
      T.3  cargo build --target wasm32-...     -- gui still wasm32-clean (the
                                                    rfd async path is what we
                                                    rely on for web later;
                                                    sync path is gated by cfg).
      T.4  Run `oovra discover` against a real temp tree, both formats.
      T.5  Visual heartbeat: cargo run -p oovra-gui (background), open
            a folder containing olibs, confirm the panel populates.
```

## 2. Execution sequence

1. L.1 — create `src/discovery.rs` with the type and function.
2. L.2 — wire into `lib.rs`.
3. L.3 — unit tests in `discovery.rs` (inside the module).
4. T.1 — `cargo test -p oovra` (should now run the new discovery
   tests in addition to the 57).
5. C.1–C.3 — add the CLI subcommand.
6. C.4 — integration test in `tests/end_to_end.rs`.
7. T.1 (again) — confirm tests still pass with the new integration
   test.
8. T.4 — exercise the CLI binary against a real temp tree manually
   (the integration test already covers the library path).
9. G.1 — add `rfd` to `gui/Cargo.toml`.
10. G.2–G.3 — code the GUI panel.
11. G.4 — adjust serde skipping for new state.
12. T.2 + T.3 — `cargo test -p oovra-gui` + wasm32 build.
13. T.5 — `cargo run -p oovra-gui` background, visual heartbeat.
14. Update sprint-tests/*.md + agent-tasks; close sprint.

## 3. Execution details (the things that matter)

- **`walkdir` usage.** Use `WalkDir::new(root).max_depth(N)` when N
  is given; otherwise `WalkDir::new(root)`. Filter to
  `entry.file_type().is_dir() && entry.file_name() == "olib"`.
  Count `.md` files via a non-recursive `std::fs::read_dir` inside
  the matched dir (the README explicitly says non-recursive count).
- **`discover` errors.** A non-existent root returns
  `OovraError::FileNotFound(root)`. A read failure mid-walk is
  logged via `eprintln!` and the walk continues (don't crash the
  whole discovery on one unreadable subtree).
- **JSON output format.** `serde_json::to_string(&Vec<DiscoveredOlib>)`
  — that requires `DiscoveredOlib: Serialize`. Derive Serialize +
  Deserialize on the struct (Deserialize for round-tripping in
  tests).
- **GUI `Open folder…` on the main thread.** `rfd::FileDialog`'s
  sync API is fine on native (it pops a system dialog, blocks until
  the user picks or cancels). We're not bothering with async here
  for native; the WASM path will use `rfd::AsyncFileDialog` in a
  later sprint when the file-system-access shim lands.
- **Serde skipping.** Mark `discovered`, `loaded_library`, and
  `selected` with `#[serde(skip)]`. Persist only `roots` (the list
  of folders the user has opened — for "recent folders" later).

## 4. Acceptance criteria for s1

- Library: `discover()` returns the expected list for a known temp
  tree; respects `max_depth`; non-existent root errors cleanly.
- CLI: `oovra discover .` prints a human summary; `oovra discover .
  --format json` prints valid JSON.
- GUI: clicking **Open folder…** pops a system file picker; picking
  a folder populates the olib sidebar; selecting an olib loads it
  and shows its atoms in the central panel with ids and meta.
- All existing test counts still pass (57 oovra + 1 gui).
- `cargo build --target wasm32-unknown-unknown -p oovra-gui` still
  succeeds (rfd's wasm32 surface differs but should compile; if not,
  gate the `pick_folder` call behind `cfg(not(target_arch = "wasm32"))`
  for s1 and surface a "open a folder via WASM" path in a later
  sprint).

## 5. Out of scope, deferred

- Recent folders / persisted root list beyond raw `roots: Vec<PathBuf>`.
- Header tags rendering (full meta + version + kind) — s1 shows
  `id` and `meta`; richer rendering is s2/s3 work.
- Tree expansion of the filesystem (the file explorer here is
  "open folder, list its olibs" — not a full directory tree
  walker). That's a possible s3/s4 enhancement.
- WASM file picking. The async path is wired in a later sprint
  alongside the WASM filesystem shim.
