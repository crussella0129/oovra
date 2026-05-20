# Sprint s3 — Research Report

**Date:** 2026-05-20
**Sprint goal:** Add the live autocompose canvas — multi-select
library components, drag-reorder them, see the rendered prompt
update in real time, and save the result as a new compound back
into the active olib.

Substrate from prior sprints carries forward: olib discovery (s1),
component listing (s2), the file-based create flow (s2). s3 is a
new central-panel mode that sits next to the editor.

## 1. Library surface — already complete

The CLI-first rule says the library covers what the canvas needs
*before* the canvas does. It does:

- `oovra::render::render_text(&[&PromptElement]) -> Result<String>`
  (src/render.rs:169) — composes a list of elements into the
  pastable prose form. **This is the live preview.** No new code
  needed.
- `oovra::render::compose(ComposeRequest) -> Result<PromptElement>`
  (src/render.rs:91) — builds a proper compound (header + body
  with delimited sub-elements) from a `Library` + an ordered list
  of `(id, Option<version>)` inputs. **This is the save-as-
  compound path.**
- `oovra::Library::load_with(path, opts)` — already used in s1/s2
  to load the active olib; the canvas reuses it.
- `oovra::write(&PromptElement, &Path)` — writes the composed
  PromptElement to `<olib>/<output_id>.md` (already validates in
  memory before any byte hits disk).

The CLI command `oovra compose` already wraps these for terminal
use. The canvas is purely a GUI presenter over the same calls.

## 2. Drag-reorder crate — pinned

[`egui_dnd`](https://crates.io/crates/egui_dnd) 0.15.0 (released
2026-03-26) depends on `egui ^0.34.0` — compatible with our
`egui = "0.34"`. Its API for the use case we have:

```rust
use egui_dnd::dnd;

dnd(ui, "canvas_list").show_vec(&mut self.canvas.order, |ui, id, handle, _state| {
    handle.ui(ui, |ui| {
        ui.label("≡"); // drag handle glyph
    });
    ui.label(format!("· {id}"));
});
```

`show_vec` mutates the vector in place when the user drags an
item — exactly the shape the canvas needs (the canvas state is a
`Vec<String>` of selected ids, kept in display order).

Sources:
- [egui_dnd on crates.io](https://crates.io/crates/egui_dnd)
- [egui_dnd docs.rs](https://docs.rs/egui_dnd/latest/egui_dnd/)
- [hello_egui demo / examples](https://github.com/lucasmerlin/hello_egui)
- [TechOverflow minimal example (2025)](https://techoverflow.net/2025/09/25/rust-egui-minimal-drag-n-drop-example-using-egui-dnd/)

## 3. UX shape — settled

### Tabs in the central panel

`Editor` and `Canvas` tabs at the top of the central panel.
Selecting a Library Component still opens the editor regardless
of tab (so navigation isn't gated by tab) — the tab just decides
which view is visible.

```
Component Editor
[ Editor ] [ Canvas ]
─────────────────────────────────────────────────
<editor view OR canvas view, depending on tab>
```

### Multi-select via per-row checkbox

Library Components rows gain a checkbox on the left:

```
☐  ·  role-declaration
☑  ·  tone-direct          <- in the canvas
☐  ▣  coding-agent         <- compound; can also be a canvas input
☑  ·  numbered-sprints
```

Click on the checkbox toggles canvas inclusion (and updates the
ordered `Vec<String>` accordingly — append on add, remove preserving
the order of the others). Click on the rest of the row opens the
component in the editor (existing s2 behavior). Two independent
click targets.

### Canvas view

```
Canvas — n component(s) selected
  Order (drag ≡ to rearrange):
    ≡ · tone-direct
    ≡ · numbered-sprints
  Live preview:
  ┌──────────────────────────────────────────┐
  │ <render_text output, updated every frame>│
  └──────────────────────────────────────────┘
  Save as compound:
    output id: [my-compound          ]
    [Save into the active olib]
```

Live preview is `render_text` recomputed each frame from the
ordered selection. Cheap (small atoms, short bodies); no caching
needed.

Save-as-compound uses `render::compose` with version pins set to
each input's *current* version in the library. The resulting
`PromptElement` is written to `<active_olib>/<output_id>.md` via
`oovra::write`. On success, the canvas's selection persists; the
discovery counts in the sidebar update lazily on the next folder
re-open (we don't refresh discovery automatically — the new
compound just appears next time the user clicks the olib).

Actually — a small touch: after a successful save, refresh the
loaded library so the new compound shows up in the components
column without an explicit reload. One extra
`Library::load_with` call; minor cost, big UX win.

## 4. Risks and constraints

- **render_text on compounds.** `render_text` recurses through
  compound bodies for the H2-per-leaf output. Compounds in the
  canvas selection therefore expand naturally. Confirmed by
  inspecting `render_for_paste` in src/render.rs.
- **Mixing atoms and compounds in the canvas.** Allowed —
  `ComposeRequest::inputs` accepts any element id in the library.
  The resulting compound has a `body_level` of
  `max(input.body_level) + 1`, computed by the existing compose
  logic.
- **Borrow-checker grief.** Canvas state mutation (selection
  toggling, drag-reorder) interleaved with renders that read the
  loaded library risks the same borrow pattern we hit in s1's
  `render_olib_list`. Same fix: snapshot the per-frame slice into
  small owned vecs before mutating self.
- **`#[allow(deprecated)]` on `App::ui`** carries forward — the
  egui Panel-alias migration is still on the roadmap (s6).
- **WASM**: the canvas works exactly the same on wasm32 *at
  compile time*; at runtime, no save-as-compound until the WASM
  filesystem shim lands (s5). For s3, the wasm32 build's only
  responsibility is to compile clean.

## 5. References

- `oovra::render` — `src/render.rs` (render_text, compose,
  ComposeRequest)
- `oovra::Library::load_with` — `src/library.rs`
- [`egui_dnd`](https://crates.io/crates/egui_dnd) — 0.15.0
- s1/s2 research reports for the layout patterns and the
  CLI-first reasoning that still applies.
