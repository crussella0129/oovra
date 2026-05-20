# Sprint s1 — Research Report

**Date:** 2026-05-19
**Sprint goal:** Add **olib autodiscovery** — a `Library::discover`
library function, an `oovra discover` CLI subcommand, and a GUI panel
that opens a folder and lists discovered olibs with their atom counts.

This sprint is CLI-first per [`../../../CLAUDE.md`](../../../CLAUDE.md):
library → CLI → GUI, in that order.

## 1. Existing project surface used by s1

- [`oovra::library::Library::load_with(path, opts)`](../../../src/library.rs)
  already parses one olib directory into an in-memory library. s1
  adds the discovery layer *around* it: find olib dirs first, then
  optionally load any of them via this existing function.
- `walkdir = "2"` is already a workspace dependency (used by
  `oovra::migrate`); reuse it instead of adding a new tree-walker.
- `looks_like_oovra_file` in `element.rs` lets us distinguish
  oovra-headered `.md`s from plain ones if a richer count is wanted
  later. For s1 we count `.md` files directly.

## 2. File-picker crate — confirmation

`rfd` ([crates.io/crates/rfd](https://crates.io/crates/rfd)) is the
right choice: cross-platform native dialogs, async **and** sync APIs,
supports both desktop and WASM (the async API works in browsers via
the File System Access API and falls back to `<input type="file">`).
The `woelper/egui_pick_file` reference repo demonstrates the
integration pattern with eframe.

For s1's native scope, the sync API is fine:
`rfd::FileDialog::new().pick_folder()` returns `Option<PathBuf>`. The
async path becomes relevant in the future WASM-features sprint.

Alternative considered: [`egui-file-dialog`](https://crates.io/crates/egui-file-dialog)
— a pure-egui dialog that looks the same on every OS. Rejected for
s1 because we want the native OS dialog (familiar to users); we can
switch later if uniformity wins out.

## 3. egui layout pattern for the panel split — pinned

`eframe::App` in 0.34.2 requires `App::ui(&mut self, ui, frame)`; the
old `App::update(&mut self, ctx, frame)` is **deprecated**. The
runtime gives us a `Ui` already attached to the central area, so the
old pattern of `egui::SidePanel::left(...).show(&ctx, ...)` is also
deprecated.

The current pattern is **`show_inside(ui, ...)`** — panels nest
inside the given `Ui`:

```rust
fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("toolbar").show_inside(ui, |ui| {
        if ui.button("Open folder…").clicked() { /* rfd */ }
    });
    egui::SidePanel::left("olibs")
        .resizable(true)
        .default_width(220.0)
        .show_inside(ui, |ui| {
            // olib list
        });
    egui::CentralPanel::default().show_inside(ui, |ui| {
        // selected olib's atoms / details
    });
}
```

Panel ordering: panels must be added before the central panel; the
first added is outermost. See [egui::containers::panel docs](https://docs.rs/egui/latest/egui/containers/panel/index.html).

## 4. Discovery semantics — what counts as an "olib"

The minimal definition for s1 is **a directory literally named
`olib`** (case-insensitive on Windows for file-system parity, but the
test path will use lowercase). A future option `--name <NAME>` can
broaden this to any directory name, but s1 hard-codes `olib`.

Rationale: matches the convention `oovra create --olib` writes to,
and avoids false positives from any directory that happens to
contain `.md` files. If a user wants discovery of arbitrary
`.md`-bearing directories they can pass that root directly to
`Library::load_with`.

## 5. Discovery shape — what `discover` returns

For s1 keep it tight: return paths and a cheap count.

```rust
pub struct DiscoveredOlib {
    pub path: PathBuf,
    pub md_count: usize, // number of `.md` files DIRECTLY inside `path` (non-recursive)
}

pub fn discover(
    root: &Path,
    max_depth: Option<usize>,
) -> Result<Vec<DiscoveredOlib>>;
```

- `max_depth = None` walks the tree without a depth limit. `Some(N)`
  bounds the walk for huge trees (`N` measured in directory depth
  from `root`, with `root` itself being depth 0).
- `md_count` is the cheap count — just counts files matching `*.md`
  in each found `olib/`. It does NOT parse them as Oovra files
  (that would require reading every file). The GUI can call
  `Library::load_with` for the selected olib to get atom/compound
  breakdowns.

Tested behaviors (planned, recorded in test-plan.md):

- A folder named `olib` directly under `root` is discovered.
- A folder named `olib` nested two levels deep is discovered.
- `max_depth = Some(1)` finds the shallow one but not the deep one.
- A folder named `Olib` (mixed case) is discovered on Windows; on
  Linux the test asserts only the lowercase. Lowest-common-denominator
  test: include both names — assertion is on the path *set*
  containing the lowercase one.
- A `.md` file directly under `root` is *not* an olib (only dirs are
  candidates).

## 6. CLI surface — `oovra discover`

```
oovra discover <ROOT> [--max-depth N] [--format human|json]
```

- ROOT — required positional; the directory to walk.
- `--max-depth N` — optional; defaults to no limit.
- `--format` — `human` (default, pretty multiline) or `json`
  (structured `[{path, md_count}, ...]`) for agent integrations.

Human output sketch:

```
Discover ./projects  (max depth: unlimited)
  ✓ ./projects/foo/olib            (12 .md)
  ✓ ./projects/bar/drafts/olib     (4 .md)
  2 olib(s) found.
```

JSON output: a single line of `serde_json::to_string` over the
`Vec<DiscoveredOlib>` — easy for an agent to pipe through `jq`.

## 7. GUI surface — file explorer panel

Visual layout (s0 bootstrap → s1):

```
┌──────────────────────────────────────────────────┐
│ [Open folder…]                                   │  ← top toolbar
├────────────┬─────────────────────────────────────┤
│ olibs:     │ selected olib: ./drafts/olib        │
│  ▸ olib(5) │   atoms (5):                        │  ← central
│  ▸ olib(12)│     • numbered-sprints              │
│            │       "" (meta empty)               │
│            │     • tone-direct                   │
│            │       "" (meta empty)               │
│            │     • ...                           │
├────────────┴─────────────────────────────────────┤
│ s0 probe still here for now (collapsible later)  │
└──────────────────────────────────────────────────┘
```

The s0 bootstrap probe (live `is_kebab_case` / `slugify`) stays for
this sprint — it's small, useful for visual diff between sprints,
and harmless. Later sprints can collapse or remove it.

## 8. Risks and notes

- **`walkdir` and symlinks.** Default `walkdir` does not follow
  symlinks; that's the safe default. We don't override it.
- **`rfd` on first run.** First `rfd::FileDialog::new().pick_folder()`
  on Windows may pop a focus toast — not blocking, just noted.
- **Discovery cost on huge trees.** Pass `--max-depth 3` (or
  similar) when scanning broad roots. The library default is
  unlimited because most prompt libraries are shallow.
- **No GUI-only library code.** All filesystem walking and counting
  is in `oovra::discovery`; the GUI imports and calls it the same
  way the CLI subcommand does. This is the CLAUDE.md CLI-first rule
  in practice.

## 9. References

- [`rfd` on crates.io](https://crates.io/crates/rfd)
- [`egui-file-dialog` on crates.io](https://crates.io/crates/egui-file-dialog) (considered, not chosen)
- [`woelper/egui_pick_file`](https://github.com/woelper/egui_pick_file) — eframe + rfd integration reference
- [egui `containers::panel` docs](https://docs.rs/egui/latest/egui/containers/panel/index.html)
- [egui SidePanel discussion #469](https://github.com/emilk/egui/discussions/469) — panel ordering and sizing
- [`walkdir` on crates.io](https://crates.io/crates/walkdir) — already a workspace dep
- `eframe::App` trait docs ([docs.rs/eframe/0.34.2/eframe/trait.App.html](https://docs.rs/eframe/0.34.2/eframe/trait.App.html))
  confirming `ui` is required, `update` deprecated.
