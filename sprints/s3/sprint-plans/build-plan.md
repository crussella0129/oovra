# Sprint s3 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Sprint goal: ship the live autocompose canvas in `oovra-gui`,
sitting next to the Component Editor as a second tab in the central
panel. Multi-select Library Components via per-row checkbox,
drag-reorder them, see the live `render_text` preview, save the
ordered selection as a new compound back into the active olib.

Source: [`../sprint-research/research-report.md`](../sprint-research/research-report.md).

CLI-first per [`/CLAUDE.md`](../../../CLAUDE.md): the library
already has everything we need (`render::render_text`,
`render::compose`, `Library::load_with`, `oovra::write`). The CLI
already wraps these via `oovra compose --text` and `oovra compose
-o`. s3 is therefore purely a GUI sprint.

## 1. Sub-schema (elementary)

```
s3 = "live autocompose canvas in oovra-gui"
│
├── G. GUI (only chunk; lib + CLI are already complete)
│     G3.1  Add `egui_dnd = "0.15"` to gui/Cargo.toml.
│     G3.2  Add `canvas` module in gui/src/canvas.rs:
│            - pub struct CanvasState {
│                  /// Ids included in the canvas, in display order.
│                  pub order: Vec<String>,
│                  /// Output id for save-as-compound.
│                  pub output_id: String,
│                  /// Last save status / error.
│                  pub status: String,
│              }
│            - impl CanvasState {
│                  fn toggle(&mut self, id: &str) { ... }
│                  fn contains(&self, id: &str) -> bool { ... }
│                  fn live_preview(&self, lib: &Library) -> Result<String>
│                      (calls render::render_text over resolved elements)
│                  fn save_as_compound(&self, lib: &Library, olib_dir: &Path)
│                      -> Result<PathBuf>
│                      (calls render::compose then oovra::write)
│              }
│     G3.3  Extend OovraApp:
│            - canvas: CanvasState
│            - view: enum CentralView { Editor, Canvas }
│              (drives which tab is active)
│     G3.4  Library Components row gains a leading checkbox:
│            - render_atom_list iterates atom_index;
│              each row: `ui.horizontal(|ui| {
│                  let mut included = self.canvas.contains(&id);
│                  if ui.checkbox(&mut included, "").changed() {
│                      self.canvas.toggle(&id);
│                  }
│                  // existing selectable_label opens editor
│              })`
│            - Click on the checkbox toggles canvas inclusion only;
│              click on the rest of the row still opens the editor.
│     G3.5  Central panel becomes tabbed:
│            - Above the heading, a row of two selectable_labels
│              ("Editor" / "Canvas") sets self.view.
│            - Below the heading + version line + separator, render
│              either render_editor or render_canvas.
│     G3.6  render_canvas:
│            - "Canvas — N component(s) selected"
│            - egui_dnd::dnd(...).show_vec(&mut canvas.order, |ui,id,handle,_| {
│                  handle.ui(ui, |ui| ui.label("≡"));
│                  ui.label(format!("· {id}"));
│              })
│            - Separator
│            - Live preview, computed each frame via
│              CanvasState::live_preview. Render in a read-only
│              multi-line TextEdit (monospace, big) so the user can
│              scroll / select / copy.
│            - Separator
│            - Save form: text_edit_singleline(&mut canvas.output_id)
│              + button "Save into active olib". On click, call
│              save_as_compound, refresh the loaded library on
│              success, append a status line.
│     G3.7  Bring the new compound into the components list after
│            a successful save by re-running load_selected_olib for
│            the active olib (so it shows up next to the atoms).
│
└── T. Test + verification
      T3.1  cargo test -p oovra            -- 64 PASS (regression).
      T3.2  cargo test -p oovra-gui        -- new canvas tests + s1/s2 carry.
      T3.3  cargo build --target wasm32-...
      T3.4  cargo run -p oovra-gui (visual heartbeat).
```

## 2. Execution sequence

1. G3.1 — `gui/Cargo.toml` adds `egui_dnd = "0.15"`.
2. G3.2 — write `gui/src/canvas.rs` with `CanvasState` and methods.
   Include unit tests:
   - canvas_toggle_idempotent
   - canvas_live_preview_matches_render_text
   - canvas_save_as_compound_round_trips
3. G3.3 — extend `OovraApp` fields, `Default`, and serde-skip the
   per-run canvas/view bits (persist nothing canvas-side this
   sprint — keep it simple).
4. G3.4 — rewrite `render_atom_list` to add the checkbox column.
5. G3.5 — wrap the central panel's heading area in a tab row;
   route to `render_editor` or `render_canvas` based on `self.view`.
6. G3.6 / G3.7 — implement `render_canvas` with egui_dnd, the
   preview, the save form; on save success, refresh the loaded
   library.
7. T3.1 + T3.2 + T3.3 — workspace tests + wasm32 build.
8. Kill running gui, T3.4 visual heartbeat.
9. Sprint docs + agent-tasks update + commit.

## 3. Execution details

- **Canvas state lifetime.** `CanvasState` doesn't borrow from
  `Library`; it stores ids as owned strings. Resolution happens
  per-call against the currently loaded library. If the user
  changes olibs, the canvas selection that no longer resolves
  silently degrades — entries whose ids aren't in the new library
  are *kept* in the order vec but render as `(missing: <id>)` in
  the canvas view. They can still be removed by toggling.
- **Drag-reorder borrow.** `dnd(...).show_vec` takes
  `&mut Vec<String>` — fine, we have one. The rendering closure
  reads `id`; we don't need to touch `self` inside it.
- **Live preview cost.** `render_text` over a handful of small
  atoms is microseconds — re-running every frame is fine. If
  later sprints add huge compounds this becomes a dirty-flag
  optimization, but YAGNI for s3.
- **Save semantics.** Output `output_id` must be kebab-case; we
  rely on `oovra::write`'s built-in validation to reject bad ids
  rather than re-implementing the check. The error surfaces in
  the canvas status string.
- **Compound version pin.** `ComposeRequest::inputs` uses
  `Option<String>` for the version pin. We set it to `Some(<the
  library's current version of that input>)` so the recorded
  recipe is exact at compose time. Future changes to the input's
  version don't silently retroactively rebind this compound.
- **`egui_dnd` interplay with the rest of egui.** The crate is
  drop-in; no `Context` configuration needed beyond importing
  `dnd`. Verified against the 0.15.0 docs.

## 4. Acceptance criteria for s3

- GUI: the central panel has Editor / Canvas tabs; clicking a
  checkbox in Library Components adds/removes the component from
  the canvas. The Canvas tab shows the selected components in a
  drag-reorderable list, a live preview that re-renders from the
  current order, and a Save form.
- Save into an active olib produces a parseable compound file at
  `<olib>/<output_id>.md` and the new compound appears in the
  Library Components column (▣ glyph) after save.
- All existing tests pass. `cargo test -p oovra-gui` includes the
  new canvas tests.
- `cargo build --target wasm32-unknown-unknown -p oovra-gui` still
  clean.

## 5. Out of scope, deferred

- Multi-select via Shift/Ctrl on the row (instead of the
  checkbox) — UX nicety, post-s3.
- Persisting canvas state across runs — the canvas is a working
  surface, not a project document. Future sprint can address.
- Editing an existing compound's recipe from the canvas (drag
  in / out of an opened compound) — semantically a "compound
  editor" feature, scope-creeps into s4's recipe-edit territory.
- The `--text-with-tags` CLI flag floated in research — keep CLI
  surface lean for now; only add if the use case crystallizes.
