# Sprint s7 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Sprint goal: surface each component's header tags (version + a
truncated meta) inline in the Library Components tree, so the user
can "see the tags from the Oovra header without opening" — a direct
item from the original GUI vision.

Display-only; no library or schema change.

## Tasks

1. `gui/src/app.rs::render_component_node`: in the per-row scoped
   borrow of `self.loaded`, also read `header.version` and
   `header.meta` for the node's id.
2. New free fn `tag_labels(ui, version, meta)`: renders `v<version>`
   (dimmed) and, when meta is non-empty, a `— <meta>` suffix
   (dimmed, truncated past 40 chars with an ellipsis).
3. Call `tag_labels` at the end of both the atom row and the
   compound row (after the selectable label / collapsing header).

## Verification (display-only; no new tests)

- `cargo clippy --workspace --all-targets` → clean.
- `cargo test --workspace` → 104, unchanged.
- `cargo build --target wasm32-unknown-unknown -p oovra-gui` → PASS.
- WSL Ubuntu `cargo test -p oovra` → parity.
- `cargo run -p oovra-gui` → each tree row shows its version and a
  short meta inline, dimmed.

## Out of scope

Tags in the canvas list, tooltips, or full-meta-on-hover — possible
later polish. This sprint is just the tree rows.
