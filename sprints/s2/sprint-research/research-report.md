# Sprint s2 — Research Report

**Date:** 2026-05-20
**Sprint goal:** Embed an oovra-particle editor in `oovra-gui` so the
user can author / revise atoms without leaving the app. Add a small
CLI mirror (`oovra inspect`) for the same read step.

This sprint is feature work on infrastructure that's already in
place; the ecosystem research from s0 and the layout pattern from s1
both still apply. Research here is short — just confirming the
specific APIs and settling the edit/save semantics.

## 1. Existing library surface that already does the heavy lifting

The CLI-first rule from [`/CLAUDE.md`](../../../CLAUDE.md) says the
library covers what the editor needs before the editor exists. It
does:

- [`oovra::parse_file(&Path)`](../../../src/element.rs) — load and
  validate an existing atom into a `PromptElement`.
- [`oovra::write(&PromptElement, &Path)`](../../../src/element.rs) —
  serialize an in-memory element back to disk; validates in memory
  before any byte hits the filesystem (so a bad edit can't produce
  an unparseable file).
- [`oovra::PromptElementHeader`](../../../src/header.rs) — direct
  field access for `id`, `name`, `version`, `meta`. Editing these
  in memory then calling `oovra::write` is the save path.
- [`oovra::header::is_kebab_case`](../../../src/header.rs) and
  [`is_valid_semver`](../../../src/header.rs) — drive
  live-validation hints in the editor (the save button should
  refuse on invalid id or version, with a clear status line).

No new library functions are needed for s2 — the editor is a
presenter over this existing surface.

## 2. egui multi-line TextEdit — pattern

The editor's body field is a multi-line, monospaced, resizable
text area. The canonical pattern in egui 0.34:

```rust
let response = ui.add(
    egui::TextEdit::multiline(&mut self.editor.body)
        .desired_width(f32::INFINITY)
        .desired_rows(20)
        .font(egui::TextStyle::Monospace),
);
if response.changed() {
    self.editor.dirty = true;
}
```

`response.changed()` flips the dirty flag so the Save button can
show "Save *" or be disabled when there's nothing to save.

For the short single-line fields (version, meta) we use
`ui.text_edit_singleline(&mut field)`.

## 3. Save semantics — settled

The editor edits an **atom in an olib** (compounds are out of scope
for s2; the editor refuses to open one with a status message). Save
flow:

1. The editor holds: `path: PathBuf`, `id: String` (read-only in
   s2), `name: String`, `version: String`, `meta: String`,
   `body: String`, `dirty: bool`.
2. On Save click: validate `is_valid_semver(&version)`; if bad,
   set status and bail.
3. Build a `PromptElementHeader { kind: Atom, id, name, version,
   meta, ..None }` and a `PromptElement::new(header, body)`; call
   `oovra::write(&element, &path)`. `write` re-validates the
   in-memory serialized form before touching the disk, so any
   inconsistency (e.g. empty body) errors clean.
4. On success, status: "Saved <path>"; `dirty = false`.
5. On failure, status: `"Save failed: {e}"`; the on-disk file is
   unchanged (per `write`'s in-memory validation guarantee).

**Why `id` is read-only this sprint.** Changing `id` changes the
canonical filename inside the olib. That's a rename, which is a
separate semantic operation (and could compose with `compose`'s
recipes that reference the old id). A future "rename / new-version"
sprint owns that flow; s2 stays narrow.

## 4. Layout — three columns

Building on s1's two-column shape:

```
┌─────────────────────────────────────────────────────────────────┐
│ [Open folder…] [New atom]   <status line>                       │
├──────────┬───────────────────────┬──────────────────────────────┤
│ olibs    │ atoms (selected olib) │ editor (selected atom)       │
│  • foo   │  • role-declaration   │  id:      role-declaration   │
│  • bar   │  • tone-direct        │  name:    [          ]       │
│          │                       │  version: [1.0.0     ]       │
│          │                       │  meta:    [          ]       │
│          │                       │                              │
│          │                       │  body:                       │
│          │                       │  ┌──────────────────────┐    │
│          │                       │  │ You are a precise... │    │
│          │                       │  └──────────────────────┘    │
│          │                       │  [Save *]                    │
└──────────┴───────────────────────┴──────────────────────────────┘
```

Egui SidePanels stack: two `SidePanel::left` calls in order produce
two left columns (first outermost), CentralPanel is innermost. The
s0 kebab/slugify probe moves into a footer `TopBottomPanel::bottom`
collapsing section.

## 5. `oovra inspect` — the CLI mirror

Read-only counterpart to the editor's "load and show" step. The
agent host (CLI-only install) gets the same introspection the GUI
shows on click.

```
oovra inspect <FILE> [--format human|json]
```

Human output:

```
Inspect ./olib/role-declaration.md
  id        role-declaration
  name      role-declaration
  kind      Atom
  version   1.0.0
  meta      Identity statement for a coding agent
  body      2 line(s), 102 chars
```

JSON: the full `PromptElementHeader` plus `body_lines` and
`body_chars`. One-line `serde_json::to_string` for easy `jq`.

`inspect` differs from `decompose`: decompose is *recipe-oriented*
(shows a compound's inputs), inspect is *element-oriented* (header
+ body summary) and works on atoms and compounds alike.

## 6. Risks / notes

- **No new library code → no new lib tests.** s2's library impact
  is zero new modules. The integration tests run the existing
  `parse_file` / `write` surface implicitly via the gui smoke test.
- **Editor opens an atom only.** Compound files are surfaced in
  the atom list but selecting one shows "compounds are not editable
  yet (s3+)" instead of opening the editor.
- **No persisted editor state.** Edits live in the editor struct
  for the session; closing the window before saving loses them.
  An "auto-save draft" feature is post-s2.
- **Conflict with the still-running s1 GUI window.** Windows locks
  the .exe; the s2 GUI rebuild requires the s1 window to be closed
  first. We kill it as part of the Build Phase.

## 7. References

- `oovra::element::write` — `src/element.rs`, the canonical
  in-memory-validating writer.
- `oovra::header` validators — `src/header.rs`.
- egui `TextEdit::multiline` —
  [`docs.rs/egui/latest/egui/widgets/text_edit/struct.TextEdit.html`](https://docs.rs/egui/latest/egui/widgets/text_edit/struct.TextEdit.html).
- s1's research-report.md still holds for the panel APIs and the
  rfd integration; not duplicated here.
