# Demo 4 — Structural Diff Cuts Through Surface Noise

## What this demonstrates

`oovra compare` on two **composed** files of the same order does a *structural* diff — comparing the `composed_of` recipes, not the rendered bodies. This surfaces "what really changed" between two compositions even when the on-disk forms differ in dozens of lines (timestamps, body prose, version strings, etc.).

## Steps that were run

1. **Library v1** (`library-v1/`): four atomics, all at `version = "1.0.0"`.
2. **Compose v1** (`pairing-v1.md`): order-1 composition of those four atomics, named `pairing-prompt`.
3. **Library v2** (`library-v2/`): same four atomics, but two of them are bumped:
   - `role-statement`: `1.0.0` → `2.0.0`, plus a body edit (`Stay grounded` → `Stay grounded in the file:line cited by the user, ...`)
   - `format-rules`: `1.0.0` → `1.1.0` (no body edit)
4. **Compose v2** (`pairing-v2.md`): same input IDs, same name, same order — composed against the bumped library.
5. **`oovra compare pairing-v1.md pairing-v2.md`** — captured in `compare-output-human.md`.
6. Same diff via `--format=json` — captured in `compare-output-json.md`.

## Folder layout

```
04-structural-diff/
├── library-v1/                     ← four atomics at v1.0.0
├── library-v2/                     ← same atomics, two bumped (role-statement → 2.0.0, format-rules → 1.1.0)
├── pairing-v1.md                   ← composed against library-v1
├── pairing-v2.md                   ← composed against library-v2 (same recipe, bumped inputs)
├── compare-output-human.md         ← `oovra compare` human-readable
└── compare-output-json.md          ← `oovra compare --format=json` for agents
```

## Why structural diff matters

A naive byte-level diff between `pairing-v1.md` and `pairing-v2.md` shows **26 lines of difference**:
- The outer file's `generated_at` timestamp (different by milliseconds)
- The version strings inside two `composed_of` table entries
- Two version strings inside the embedded sub-element headers in the body
- The entire body of `role-statement` (because the prose was rewritten)

Most of those differences are noise. They tell you the files were generated at different times against different libraries; they don't tell you *what kind of change happened* at the recipe level.

The structural diff answers the recipe-level question directly:
- **0 inputs added** (both compositions have the same 4 elements)
- **0 inputs removed**
- **2 inputs version-changed** (role-statement and format-rules)
- **`recipes_equal = false`** (the recipes differ — but only in version pins, not in structure)

This is the diff you actually want when reviewing a prompt change in a PR, or when deciding whether two prompts in your library are "the same composition" for deduplication purposes.

## How the implementation works

`structural_diff()` in `src/diff.rs`:

1. Pull the `composed_of` arrays out of both headers — these are `Vec<InputRef>` where each `InputRef` is `{ id, version }`.
2. Index each by ID into a `HashMap<&str, &str>` (id → version).
3. Walk B's inputs:
   - If A has the same ID at a *different* version → record a `VersionChange`
   - If A doesn't have this ID → record `added`
4. Walk A's inputs:
   - If B doesn't have this ID → record `removed`
5. Bundle into a `StructuralDiff` with `added`, `removed`, `version_changed`, and `recipes_equal` (true iff all three lists are empty).

## Limitation (intentional, v0.1)

Structural diff treats `composed_of` as a **set keyed by ID**, not a sequence. Reordering inputs (e.g. role-statement, safety-fence, X → safety-fence, role-statement, X) doesn't show up as a diff. This is intentional for v0.1 because order-of-inputs is rendering-only metadata in the current single-render-mode design. v0.2 might add a `--ordered` flag for cases where you care about position.

## See also

- [reference/command-compare.md](../../reference/command-compare.md) — full reference for the 4-way dispatch (content, structural, order-mismatch, atomicity-mismatch)
- [reference/schema.md § composed_of](../../reference/schema.md#the-four-conditional-fields-composed-elements-only) — the recipe field structural diff operates on
- [reference/command-compose.md § --re-render](../../reference/command-compose.md#mode-3---re-render-path--regenerate-an-existing-composed-files-body) — re-rendering uses the same recipe but with strict pin enforcement
