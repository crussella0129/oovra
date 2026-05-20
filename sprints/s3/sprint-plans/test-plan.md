# Sprint s3 — Test Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Counterpart to [`build-plan.md`](./build-plan.md). s3 is GUI-only;
library coverage carries forward unchanged. The new tests live in
the gui crate (the `canvas` module).

## 1. Unit tests

### Library — none new

`render::render_text`, `render::compose`, `oovra::write`, and
`Library::load_with` are all already covered by the 24 integration
tests in `tests/end_to_end.rs`. No new lib code in s3 → no new
lib tests.

### GUI — `oovra-gui::canvas`

In `gui/src/canvas.rs::tests`:

- **U3-1 — `canvas_toggle_is_idempotent`.** Empty canvas →
  toggle id `a` → order = `[a]`. Toggle `a` again → empty. Toggle
  `a`, `b`, `c` → order = `[a, b, c]`. Toggle `b` → order =
  `[a, c]`. Catches off-by-one in the remove path.
- **U3-2 — `canvas_live_preview_matches_render_text`.** Build a
  tiny library with two atoms; toggle both into the canvas in a
  specific order; assert `canvas.live_preview(&lib)?` equals
  `render::render_text(&[&atom1, &atom2])?`.
- **U3-3 — `canvas_save_as_compound_round_trips`.** Same library,
  same canvas. Call `canvas.save_as_compound(&lib, &olib_dir)`.
  Then `parse_file(olib_dir/<output_id>.md)` and assert:
  - kind == Compound.
  - composed_of contains the expected ids in order.
  - body_level == 1 (atom inputs → level 1).
- **U3-4 — `canvas_skips_missing_ids_in_preview`.** Add an id to
  the canvas that isn't in the library; `live_preview` returns an
  error referencing the missing id (or skips it gracefully —
  decide based on render_text's existing behavior; the test
  pins whichever it is).

### s1/s2 carry-forward

- `default_app_is_constructible_and_oovra_lib_is_reachable` (s0)
- `short_path_*` (s1)
- `editor_*` (s2 — 3 tests)

Target gui-crate test count: **3 (app/short_path) + 3 (editor) + 4
(canvas) = 10**.

## 2. Integration tests

- **I3-1 — `cargo test -p oovra` still 64 PASS.** Regression.
- **I3-2 — `cargo test -p oovra-gui` 10 PASS.** New canvas tests
  bring the total to 10.
- **I3-3 — `cargo build --target wasm32-unknown-unknown -p
  oovra-gui` PASS.** `egui_dnd` must compile cleanly for wasm32 —
  if it doesn't, gate the canvas tab behind
  `cfg(not(target_arch = "wasm32"))` and surface a "canvas
  unavailable on web until s5" hint, then carry on.
- **I3-4 — workspace clippy clean.** No new warnings.

## 3. End-to-End test

- **E3-1 — Canvas round-trip in the running app.**
  `cargo run -p oovra-gui` in the background. User walkthrough:
  1. Open `C:\Users\charl\oovra-demo`, select
     `coding-agent/olib`. Components list shows
     `role-declaration` and `tone-direct` with checkboxes.
  2. Check both → switch to **Canvas** tab. The selected ids are
     in the canvas in click order. Drag-reorder them via the ≡
     handle.
  3. Live preview renders the H2-per-leaf prose form, updates
     instantly when the order changes.
  4. Enter `coding-agent-prompt` as the output id, click **Save
     into active olib**. Status: `Saved <path>`. The Library
     Components column re-loads to show the new compound
     `coding-agent-prompt` with the ▣ glyph.
  5. Run `oovra inspect C:/Users/charl/oovra-demo/coding-agent/olib/coding-agent-prompt.md`
     from a terminal to confirm: kind == Compound, composed_of
     listing the two atom inputs in the canvas-chosen order.

## 4. CI verification — DEFERRED (same posture as s0/s1/s2)

## 5. Logging conventions — same as prior sprints
