# `oovra compare` — Reference

Diffs two Oovra files. The FORWARD-DIFF operator. Dispatches on the `kind` of each input into one of three outcomes: content diff (atom vs atom), structural diff (compound vs compound), or kind mismatch (refused). The compound-vs-compound diff is **sequence-aware** in v0.2 — reordering inputs is reported as a `moved` axis.

> **v0.2 migration note** — v0.1's two-axis dispatch (order match + atomicity match) collapsed into a single kind-aware dispatch. `OrderMismatch` is gone; `AtomicityMismatch` is renamed `KindMismatch`. Structural diff gained `moved` and changed `added`/`removed` from `Vec<InputRef>` to `Vec<PositionedInput>`.

## Synopsis

```
oovra compare <A> <B>                        # human-readable
oovra compare <A> <B> --format json          # machine-readable
oovra --legacy compare <A> <B>               # accept v0.1 files (transitional)
```

## Flags

| Flag                | Default          | Purpose                                                |
|---------------------|------------------|--------------------------------------------------------|
| `<A>` (positional)  | required         | First file (the "before" side in diff terms)            |
| `<B>` (positional)  | required         | Second file (the "after" side)                          |
| `--format <FMT>`    | `human`          | `human` for colored terminal output; `json` for tools  |
| `--legacy` (global) | off              | Accept v0.1-schema files (with `order` instead of `kind`); v0.2-only by default |

---

## The dispatch table

| `A.kind`   | `B.kind`   | Result                                                                      |
|------------|------------|-----------------------------------------------------------------------------|
| `atom`     | `atom`     | **Content diff**: body unified diff + metadata change list                  |
| `compound` | `compound` | **Structural diff**: added / removed / version-changed / moved over `composed_of` |
| `atom`     | `compound` | Refused: [`KindMismatch`](./errors.md#kindmismatch)                         |
| `compound` | `atom`     | Refused: same                                                               |

In words: the two inputs must be the **same kind**. Mixed comparisons are refused cleanly.

---

## Outcome 1: Content diff (atom vs atom)

When both files are atoms, Compare produces a **content diff**: a unified-diff-style report of body differences plus a list of metadata field changes.

### What's reported

- **Field changes**: any of `name`, `version`, `meta` that differ between A and B, shown as `<field>: <A-value> -> <B-value>`
- **Body diff**: a standard unified diff (LCS-based, the same algorithm Git uses) over the body content. Lines that match are unchanged; lines beginning with `-` are removed, `+` are added, `@@` are hunk markers.

The `id` field is **not** included in the field-change list — comparing two files with different IDs is a normal use case (comparing two versions of conceptually the same element saved under different filenames).

### `--format human` output (example)

```
Compare role-statement <-> role-clone  (atoms, content diff)
  metadata changes:
    version : 1.0.0 -> 2.0.0
    meta : Who the assistant is -> Refined role statement
  body diff:
    --- role-statement
    +++ role-clone
    @@ -1 +1 @@
    -You are a senior software engineer pairing with the developer at the keyboard.
    +You are a principal software engineer pairing with the developer at the keyboard.
```

### `--format json` output (example)

```json
{
  "kind": "content",
  "a_id": "role-statement",
  "b_id": "role-clone",
  "field_changes": [
    { "field": "version", "before": "1.0.0", "after": "2.0.0" },
    { "field": "meta", "before": "Who the assistant is", "after": "Refined role statement" }
  ],
  "body_unified_diff": "--- role-statement\n+++ role-clone\n@@ -1 +1 @@\n-You are a senior software engineer ...\n+You are a principal software engineer ...\n",
  "bodies_equal": false
}
```

`bodies_equal` is true iff the bodies were byte-identical.

### Diff granularity: line-level, not word-level

Compare uses [`similar::TextDiff::from_lines`](https://docs.rs/similar/latest/similar/struct.TextDiff.html#method.from_lines), the standard LCS-based line-oriented diff. Prompt bodies are paragraph-shaped — line-level diff matches the unit a human edits in. Word-level diff produces visually noisy reports for multi-line edits; character-level is too granular for prose.

---

## Outcome 2: Structural diff (compound vs compound) — sequence-aware

When both files are compounds, Compare compares their **`composed_of` recipes** — not their rendered bodies. This is the architectural payoff of having an explicit recipe field: structural differences can be surfaced cleanly even when the bodies differ in many lines (timestamps, embedded prose updates, etc.).

In v0.2 the diff is **sequence-aware**: a reordered input is surfaced as a `moved` entry, not as a phantom add+remove. Reordering changes the rendered prompt, so a position change is a real diff axis.

### What's reported

Each input is classified along up to four axes:

- **`added`** — id present in B, absent in A. Reported with the position in B.
- **`removed`** — id present in A, absent in B. Reported with the position in A.
- **`version_changed`** — id present in both with different versions.
- **`moved`** — id present in both with the same version but at a different position in `composed_of`.

`version_changed` and `moved` are **not mutually exclusive**: an input that was both bumped *and* reordered surfaces on both lists.

`recipes_equal` is true iff all four lists are empty.

### `--format human` output (example — version bump only)

```
Compare pairing-prompt <-> pairing-prompt  (compounds, structural diff)
  version-changed inputs:
    ~ role-statement : 1.0.0 -> 2.0.0
```

### `--format human` output (example — add, remove, and reorder all at once)

```
Compare full-agent <-> full-agent  (compounds, structural diff)
  added inputs:
    + [2] format-rules @ 1.0.0
  removed inputs:
    - [0] examples-discipline @ 1.0.0
  version-changed inputs:
    ~ role-statement : 1.0.0 -> 1.1.0
  moved inputs:
    ↔ tone-direct @ 1.0.0 : pos 2 -> pos 0
```

The `[N]` brackets after `+` and `-` show the position in the respective input list. The `↔` indicates a move with the old and new positions.

### `--format json` output (example)

```json
{
  "kind": "structural",
  "a_id": "pairing-prompt",
  "b_id": "pairing-prompt",
  "added": [
    { "position": 2, "input": { "id": "format-rules", "version": "1.0.0" } }
  ],
  "removed": [
    { "position": 0, "input": { "id": "examples-discipline", "version": "1.0.0" } }
  ],
  "version_changed": [
    { "id": "role-statement", "before_version": "1.0.0", "after_version": "1.1.0" }
  ],
  "moved": [
    { "id": "tone-direct", "version": "1.0.0", "before_pos": 2, "after_pos": 0 }
  ],
  "recipes_equal": false
}
```

### Why sequence-awareness matters

In v0.1, the diff was order-blind: composing `[role, safety, tone]` then `[tone, safety, role]` produced two files that rendered different prompts but the diff reported `recipes_equal: true`. v0.2 fixes this — the reorder is a genuine semantic change because it changes the prompt the LLM sees.

The id-based dispatch still cuts through surface byte-level noise (timestamps, embedded prose updates) — you get the recipe-level meaning, plus a clean `moved` signal when input order changes.

### Duplicate-id limitation

The implementation builds id → (position, version) maps for each side. If the same id appears multiple times in a single `composed_of` array, the last occurrence wins in the maps, so moves involving duplicate-id inputs are not detected. Duplicate ids are rare (and arguably a smell) — a full LCS-based diff that handles duplicates is on the v0.3 roadmap.

---

## Outcome 3: `KindMismatch` (refused)

When one input is an atom and the other is a compound, Compare refuses with:

```
Error: Cannot compare an atom with a compound: 'A_ID' is an atom, 'B_ID' is a compound.
```

### Why refuse rather than attempt a cross-kind diff?

A cross-kind diff is ambiguous. Comparing an atom against a compound that contains it (or doesn't) raises a different question from "what changed between these two." There's no universally-right answer for a cross-kind comparison, so Compare refuses cleanly.

If you want to ask "what atoms does this compound contain", use [`oovra decompose <compound>`](./command-decompose.md) instead. If you want to compare the rendered text of a compound against an atom's body, use `compose --text` to flatten the compound first.

---

## Failure modes

Beyond the dispatch refusal listed above:

| Trigger                                       | Error                                                                                       |
|-----------------------------------------------|---------------------------------------------------------------------------------------------|
| Either file does not exist                    | [`FileNotFound`](./errors.md#filenotfound) (path is reported)                              |
| Either file fails to parse                    | The corresponding parse error ([`InvalidToml`](./errors.md#invalidtoml), [`MissingOpenDelimiter`](./errors.md#missingopendelimiter), etc.) |
| Either file is a v0.1 file without `--legacy` | [`InvalidToml`](./errors.md#invalidtoml) ("missing field `kind`"). Run `oovra migrate` or pass `--legacy`. |

---

## See also

- [schema.md](./schema.md) — what fields are diff'd
- [kind-and-delimiters.md](./kind-and-delimiters.md) — `body_level` and `depth` semantics
- [command-compose.md](./command-compose.md) — produces the files Compare diffs
- [errors.md](./errors.md) — refusal error variants in detail
- [demos/v0.1/04-structural-diff](../demos/v0.1/04-structural-diff/) — end-to-end demonstration of structural diff cutting through noise
- [demos/v0.2/01-operator-gamut](../demos/v0.2/01-operator-gamut/) — all four diff axes (added, removed, version_changed, moved) firing simultaneously, including the non-mutually-exclusive case
