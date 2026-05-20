# oovra-gui — roadmap (post-s2)

Drafted 2026-05-20 after the close of sprint s2. Lives at the
`sprints/` root because it spans sprints; each sprint that picks up
an entry below converts it into its own `sprints/sN/sprint-plans/`.

Roadmap items are ordered by **highest-leverage-next**. The agent
picks them up in this order unless the user redirects.

## What ships today (s0 → s2)

- CLI: create (--label / --olib), compose, decompose, compare,
  migrate, **discover** (s1), **inspect** (s2).
- Library: parser, library loader, header validators (kebab,
  semver, rfc3339), **discovery** module, the create primitives.
- GUI shell (s0): native + wasm32-clean eframe app linking the
  `oovra` lib crate; CLI-only install path preserved.
- GUI s1: olib autodiscovery + 2-column file explorer.
- GUI s2: 3-column layout with an embedded atom editor (open ·
  edit body / name / version / meta · save / reload). Compounds
  surface but are read-only this sprint.

The CLI is currently **ahead** of the GUI on `compose` /
`decompose` / `compare`. Per [`/CLAUDE.md`](../CLAUDE.md) CLI-first
that's the correct direction; the upcoming sprints bring the GUI
to parity.

## s3 — Live autocompose canvas

User-articulated goal from the original vision:

> a tab that you can go to that will just autocompose on the screen
> in real time so you can just select and de-select files from an
> olib, drag them around to change the order, see the tags from the
> Oovra header without opening, make a new version of a prompt

CLI-first surface:

- Library: no new functions needed — `render::render_text(&[&PromptElement])`
  already exists. The canvas is a presenter over it.
- CLI mirror: `oovra compose --text <id> <id> ...` already produces
  the rendered text. Maybe add `oovra compose --text-with-tags` or
  similar that interleaves tag annotations — TBD in s3 research.

GUI surface (the load-bearing chunk):

- New tab in the central panel: "Autocompose canvas."
- Drag-reorderable list of selected atoms (use [`egui_dnd`](https://crates.io/crates/egui_dnd)).
- Per-row inline view of the atom's `meta` / `version` ("tags")
  without opening the editor.
- Live-rendered composed body next to the list, re-rendered every
  frame via `render::render_text` over the current selection.
- "Save composed" button that calls `render::compose` to write a
  proper compound back into the olib (using
  `create::label_into_olib` semantics for the destination).

Dependencies new in s3: `egui_dnd`. Estimated complexity:
medium-high; the drag-reorder code is the bulk.

## s4 — Diff / versioning view

User goal: "make a new version of a prompt." Plus the broader value
of seeing what changed between two compositions.

CLI is already there: `oovra compare a.md b.md [--format human|json]`
emits a structural diff for compounds and a content diff for atoms.

GUI surface:

- A "Compare" pane: pick two elements (from the same olib or
  cross-olib) and render the same output `oovra compare` produces,
  with hunks colored.
- A "New version of <id>" affordance from the editor: bumps the
  semver patch, writes to `<id>-v<version>.md` or similar (TBD —
  s4 research will settle the file-naming convention vs. embedded
  version-history-in-place).

Dependencies new in s4: maybe `egui_extras` for table layout in the
diff view. Modest complexity.

## s5 — WASM file-system shim + Trunk pipeline

The whole-tool WASM story has been deferred since s0. Now is the
right time: the GUI is feature-rich enough that web users get
genuine value.

CLI: not directly affected; the CLI doesn't target WASM.

GUI surface:

- `OlibSource` trait in the gui crate with `discover`, `load`,
  `write`.
- Native impl uses `std::fs` + `oovra::Library::load_with` +
  `oovra::write` (existing behavior, factored).
- Web impl uses the File System Access API via `web-sys`, falling
  back to `<input type="file" webkitdirectory>` for browsers that
  lack FSA. Persists picked-folder handles via IndexedDB so the
  user doesn't re-pick on every reload.
- `Trunk.toml` lands; `trunk serve` + `trunk build --release`
  documented in README. CI gets a wasm32 build job (the
  push-authorization gate from s0/s1/s2 remains the user's call).

Dependencies new in s5: nothing direct on the Rust side; `trunk` is
a build-time external tool. Big diff in JS-side shape (index.html
gets fleshed out).

## s6 — egui Panel-alias migration

Carry-forward from s0/s1/s2 issue reports. egui 0.34.2 deprecated
`TopBottomPanel` / `SidePanel` aliases and `default_width` in favor
of a unified `Panel<Side>` generic and `default_size`. Migrate the
gui crate to the new API, remove the `#[allow(deprecated)]` on
`App::ui`, and lock in the egui-version range we target.

Small, mechanical sprint. Slot it wherever the rhythm permits —
could ride along with s3 if the canvas pulls in new egui APIs we
want fresh.

## s7 — Syntax-highlighted editor + Markdown preview

The s2 editor uses a plain `egui::TextEdit::multiline` with the
monospace text style. Upgrading to [`egui_code_editor`](https://crates.io/crates/egui_code_editor)
gives syntax highlighting, line numbers, and basic autocomplete. A
side-by-side rendered Markdown preview (via [`egui_commonmark`](https://crates.io/crates/egui_commonmark))
is the natural pairing.

User-facing value: writing prompts feels like writing code, which
matches what these atoms actually are.

Dependencies new in s7: `egui_code_editor`, `egui_commonmark`.

## s8 — Polish + agent-API hardening

Whatever surfaces after s3–s7 land. Likely candidates:

- A `oovra serve --watch <olib>` JSON-over-stdin daemon for tighter
  agent loops (avoids cold-start cost of repeated CLI invocations).
- "Recent folders" UI built on the existing `roots: Vec<PathBuf>`
  persistence in `OovraApp`.
- A keyboard-only mode for the GUI (Ctrl+S, Ctrl+R, arrow-keys for
  list navigation).

Slot order subject to revision as the project matures.

## Ordering rationale

s3 (autocompose canvas) is FIRST because it's the feature most
asked for in the original vision and the GUI's existing
infrastructure (olib panel + atom list + editor) is the bedrock it
sits on. s4 (diff/versioning) is second because it shares state
shape with the canvas (multi-element selection / comparison). s5
(WASM filesystem) is third because it's a discrete, finite scope
that unlocks the entire web build. s6 (egui migration) and s7
(editor polish) are quality plays; s8 catches whatever proves
load-bearing in practice.

The user's autonomy directive in
[`/feedback_oovra_autonomy_and_cli_first`](../../.claude/projects/C--Users-charl/memory/feedback_oovra_autonomy_and_cli_first.md)
governs which architectural forks warrant a stop. The candidates
in s3–s8 above are all incremental over the substrate that exists;
none of them break public API or change project layout. So the
default posture is: pick the next roadmap entry, scaffold its
sprint dir, execute. Stop only if a fork or a visual-heartbeat
moment surfaces during the work.
