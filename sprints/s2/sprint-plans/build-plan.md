# Sprint s2 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Sprint goal: embed an oovra-particle editor in `oovra-gui`. Add the
read-only CLI mirror `oovra inspect` so the same introspection the
editor performs is available to agents from the terminal.

Source: [`../sprint-research/research-report.md`](../sprint-research/research-report.md).

The CLAUDE.md CLI-first rule applies as always — but the editor is
unique in that the **CLI's equivalent of an editor is the user's own
text editor**. Anyone editing from the terminal opens the `.md` file
in their editor of choice, edits, then runs `oovra create --label`
or `--olib`. The CLI never grows an interactive editor itself. The
piece the CLI *does* gain is the read-side: `oovra inspect`.

## 1. Sub-schema (elementary)

```
s2 = "embedded particle editor + inspect CLI"
│
├── C. CLI (lands first)
│     C2.1  Add Command::Inspect(InspectArgs) to src/main.rs.
│     C2.2  InspectArgs: path (positional), format (--format,
│            default "human").
│     C2.3  run_inspect():
│            - parse_file_with(path, opts).
│            - human: header field-by-field + body line/char counts.
│            - json:  serde_json over a small struct combining
│                      PromptElementHeader + body summary.
│     C2.4  Quick manual smoke against the s1 demo tree.
│
├── G. GUI (the load-bearing chunk)
│     G2.1  Add an `Editor` struct in gui/src/editor.rs with the
│            mutable session state for one atom (path, id, name,
│            version, meta, body, dirty, status).
│     G2.2  Extend OovraApp with `editor: Option<Editor>` plus the
│            selected_atom_idx so the atoms-list selection drives
│            which file the editor holds.
│     G2.3  Restructure app.rs::ui to three columns:
│            - SidePanel::left("olibs")  — unchanged from s1.
│            - SidePanel::left("atoms")  — moves the atom list out
│              of the central panel; selecting an atom calls
│              editor::open(path).
│            - CentralPanel              — the editor view, or the
│              s0-style placeholder when no editor is loaded.
│     G2.4  Editor view:
│            - id label (read-only this sprint).
│            - name/version/meta single-line TextEdits.
│            - body multi-line TextEdit (monospace, fills available
│              height, ~20 default rows).
│            - "Save *" button (disabled when not dirty); calls
│              oovra::write on a freshly-built PromptElement.
│            - "Reload" button discards edits and re-parses the file.
│            - Live status line under the buttons.
│     G2.5  Open-atom guard: if the parsed element is a compound,
│            the editor refuses with "compounds are not editable
│            yet (s3+)" and the atoms list selection still
│            highlights but the central panel keeps a hint.
│     G2.6  Move the s0 kebab/slugify probe to a bottom
│            TopBottomPanel::bottom collapsing section so it stays
│            available without competing with the editor for the
│            central area.
│     G2.7  serde-skip the editor state; persist only what we
│            already persisted (kebab_probe, roots) plus the new
│            selected_atom_idx if we want session continuity (skip
│            for s2 — keep persistence minimal).
│
└── T. Tests + verification
      T2.1  cargo test -p oovra        -- 64 tests still pass (no
                                          new lib code; regression
                                          guard).
      T2.2  cargo test -p oovra-gui    -- editor::open round-trip
                                          test (parse + serialize)
                                          + the existing smoke.
      T2.3  cargo build --target wasm32-unknown-unknown -p oovra-gui.
      T2.4  oovra inspect smoke against the s1 demo tree's atoms
            (human + json).
      T2.5  Visual heartbeat: kill the running s1 window, cargo run
            -p oovra-gui; confirm 3-column layout; clicking an atom
            opens it in the editor; Save writes back and a re-parse
            of the file matches the edited content.
```

## 2. Execution sequence

1. C2.1–C2.3 — `oovra inspect` subcommand in `src/main.rs` (plus a
   tiny `InspectReport` struct for JSON).
2. T2.1 — `cargo test -p oovra` (sanity; CLI changes shouldn't
   regress the lib).
3. T2.4 — `cargo run -p oovra -- inspect <atom>.md` against a demo
   tree atom; eyeball the human + JSON outputs.
4. G2.1 — write `gui/src/editor.rs` with the `Editor` struct and
   `Editor::open` / `Editor::save` / `Editor::reload` methods.
5. G2.2 — extend `OovraApp` state; add `editor: Option<Editor>`.
6. G2.3–G2.6 — rewrite `app.rs::ui` to the three-column layout.
7. Kill the running s1 `oovra-gui.exe` window (Windows .exe lock).
8. T2.2 — `cargo test -p oovra-gui` builds + runs new tests.
9. T2.3 — `cargo build --target wasm32-unknown-unknown -p oovra-gui`.
10. T2.5 — `cargo run -p oovra-gui` background; verify visually
    (this is the s2 heartbeat).
11. Update `agent-tasks/` + `sprints/s2/sprint-tests/*.md`;
    commit s2.

## 3. Execution details

- **Editor module placement.** New file `gui/src/editor.rs`,
  included via `mod editor;` in `gui/src/lib.rs`. Keeps `app.rs`
  from getting unwieldy.
- **`Editor::save` does NOT call `label_in_place`.** That helper is
  for plain → headered conversion. The editor already holds a
  parsed element; we just rebuild the header + body and call
  `oovra::write` directly. Simpler, and `write` does the
  in-memory validation pass that prevents broken files on disk.
- **Don't refresh discovery after save.** The on-disk content
  changes, but the discovery results (paths, md counts) don't.
  Skipping a re-discover keeps the sidebar stable and matches what
  the user expects after a save.
- **Selection state lives on `OovraApp`, not `Editor`.** The
  atom list highlights based on `selected_atom_idx`; the editor
  state is loaded once on click and only refreshed on Reload or a
  new selection. Keeps the editor independent of list navigation.
- **`#[allow(deprecated)]` carries forward** while the egui Panel
  alias deprecations are unresolved. Recorded in the s1 test
  report; the migration is a discrete future sprint.

## 4. Acceptance criteria for s2

- CLI: `oovra inspect <atom.md>` prints the header fields plus body
  line/char counts in human format; `--format json` prints valid
  one-line JSON.
- GUI: 3-column layout; clicking an atom opens it in the editor;
  editing the body sets the dirty flag; **Save** writes back and
  a `parse_file` on the saved path matches the edited body.
  Compounds show the "not editable yet" message instead of
  opening.
- All existing tests pass: 64 oovra tests, gui smoke still PASS,
  wasm32 build still PASS.
- The running window is replaced with the s2 build for the visual
  heartbeat.

## 5. Out of scope

- Editing the element id (rename). Future "version / rename" sprint.
- Editing compound bodies (structural edits over `composed_of`).
- Persisted editor drafts across runs.
- Header field validation in real time (red highlights) beyond the
  Save-time gate. Live indicator is a quality-of-life sprint later.
- The Markdown / oovra-header syntax-highlighted view via
  `egui_code_editor`. The s2 editor uses a plain monospace
  TextEdit; syntax highlighting is a polish sprint.
