# `oovra compare` — Reference

Diffs two Oovra files. The FORWARD-DIFF operator. Dispatches on **two axes** — order match and atomicity match — into one of four outcomes: content diff, structural diff, order mismatch (refused), or atomicity mismatch (refused).

## Synopsis

```
oovra compare <A> <B>                        # human-readable
oovra compare <A> <B> --format json          # machine-readable
```

## Flags

| Flag                | Default          | Purpose                                                |
|---------------------|------------------|--------------------------------------------------------|
| `<A>` (positional)  | required         | First file (the "before" side in diff terms)            |
| `<B>` (positional)  | required         | Second file (the "after" side)                          |
| `--format <FMT>`    | `human`          | `human` for colored terminal output; `json` for tools  |

---

## The 4-way dispatch table

| `A.order == B.order`? | `A.is_atomic()` | `B.is_atomic()` | Result                                                                      |
|-----------------------|-----------------|-----------------|-----------------------------------------------------------------------------|
| no                    | (any)           | (any)           | Refused: [`OrderMismatch`](./errors.md#ordermismatch)                       |
| yes                   | atomic          | atomic          | **Content diff**: body unified diff + metadata change list                  |
| yes                   | composed        | composed        | **Structural diff**: added / removed / version-changed inputs over `composed_of` |
| yes                   | atomic          | composed        | Refused: [`AtomicityMismatch`](./errors.md#atomicitymismatch)               |
| yes                   | composed        | atomic          | Refused: same                                                               |

In words: the two inputs must be at the **same logical order** AND **the same kind** (both atomic or both composed). The error messages are specific so an agent reading them can take the right action.

---

## Outcome 1: Content diff (atomic-vs-atomic)

When both files are atomic and have the same `order` (= 0), Compare produces a **content diff**: a unified-diff-style report of body differences plus a list of metadata field changes.

### What's reported

- **Field changes**: any of `name`, `version`, `meta` that differ between A and B, shown as `<field>: <A-value> -> <B-value>`
- **Body diff**: a standard unified diff (LCS-based, the same algorithm Git uses) over the body content. Lines that match are unchanged; lines beginning with `-` are removed, `+` are added, `@@` are hunk markers.

The `id` field is **not** included in the field-change list — comparing two files with different IDs is a normal use case (comparing two versions of conceptually the same element saved under different filenames), and the IDs themselves would clutter the report.

### `--format human` output (example)

```
Compare role-statement <-> role-clone  (order 0, content diff)
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

Compare uses [`similar::TextDiff::from_lines`](https://docs.rs/similar/latest/similar/struct.TextDiff.html#method.from_lines), the standard LCS-based line-oriented diff. The reasoning:

- Prompt bodies are **paragraph-shaped**. Line-level diff matches the unit a human edits in.
- Word-level diff over multi-line bodies produces visually noisy reports where a small edit looks like a wholesale rewrite.
- Character-level diff is too granular for prose.

Line-level matches the editing granularity, which is what makes the report useful in a code-review context.

---

## Outcome 2: Structural diff (composed-vs-composed)

When both files are composed and have the same `order`, Compare compares their **`composed_of` recipes** — not their rendered bodies. This is the architectural payoff of having an explicit recipe field: structural differences can be surfaced cleanly even when the bodies differ in many lines (timestamps, embedded prose updates, etc.).

### What's reported

The recipe is treated as a **set keyed by ID** (not a sequence). For each input ID present in either A or B:

- If A has it at one version and B has it at a different version → `version_changed` entry
- If only B has it → `added`
- If only A has it → `removed`
- If both have it at the same version → no entry (no difference for this input)

`recipes_equal` is true iff all three lists are empty.

### `--format human` output (example)

Two compositions where one input was version-bumped:

```
Compare pairing-prompt <-> pairing-prompt  (order 1, structural diff)
  version-changed inputs:
    ~ role-statement : 1.0.0 -> 2.0.0
```

A more complex example with additions and removals:

```
Compare full-agent <-> full-agent  (order 1, structural diff)
  added inputs:
    + format-rules @ 1.0.0
  removed inputs:
    - examples-discipline @ 1.0.0
  version-changed inputs:
    ~ role-statement : 1.0.0 -> 1.1.0
```

### `--format json` output (example)

```json
{
  "kind": "structural",
  "a_id": "pairing-prompt",
  "b_id": "pairing-prompt",
  "order": 1,
  "added": [
    { "id": "format-rules", "version": "1.0.0" }
  ],
  "removed": [
    { "id": "examples-discipline", "version": "1.0.0" }
  ],
  "version_changed": [
    { "id": "role-statement", "before_version": "1.0.0", "after_version": "1.1.0" }
  ],
  "recipes_equal": false
}
```

### Why this matters: the structural diff cuts through surface noise

Two composed files can differ across many lines at the byte level (different `generated_at` timestamps, version strings repeated in `composed_of` and in embedded headers, body prose updated because an input was rewritten) while having **the same recipe-level meaning**. A naïve `diff` shows the byte-level noise. The structural diff answers the recipe-level question directly.

[demos/04-structural-diff](../demos/04-structural-diff/) shows the canonical example: two files that differ across 26 lines, summarized as "two inputs version-bumped, otherwise identical recipe."

### Set vs sequence: a deliberate v0.1 limitation

`composed_of` is treated as a set keyed by ID. **Reordering inputs is not reported as a diff.** If you compose `[role, safety, tone]` and then `[safety, role, tone]`, those produce different files (different body order), but their structural diff shows `recipes_equal: true`.

Rationale: in v0.1 with one render mode (`markdown-h2`), input order is rendering metadata only. The recipe-equivalent question is "do these compositions reach the same set of inputs at the same versions" — set semantics give the right answer.

If you care about positional order (e.g., for diffing across renders that ARE position-sensitive), use raw `diff` on the on-disk files instead. A `--ordered` flag for Compare is a v0.2+ consideration.

---

## Outcome 3: `OrderMismatch` (refused)

When `A.order != B.order`, Compare refuses with:

```
Error: Cannot compare elements of different orders: 'A_ID' is order N, 'B_ID' is order M. Compare requires same-order inputs.
```

### Why refuse rather than attempt a cross-order diff?

A cross-order diff is ambiguous. "Is this atomic A 'in' this composed B?" is a different question from "what changed between these two." There's no universally-right answer for a cross-order comparison, so Compare refuses cleanly and tells the user to fix the inputs.

If you actually want to ask "what atomic leaves does B contain", use [`oovra decompose <B>`](./command-decompose.md) instead.

---

## Outcome 4: `AtomicityMismatch` (refused)

A subtle case: both files are at `order = 0`, but one is hand-authored (atomic, no `composed_of`) and the other was produced by `compose([single-input])` (composed, has `composed_of` with one entry).

When `A.is_atomic() != B.is_atomic()`, Compare refuses with:

```
Error: Cannot compare an atomic element with a composed element: 'A_ID' is atomic, 'B_ID' is composed. Compare requires both inputs to be the same kind (both atomic or both composed).
```

### Why this case exists

Compose with a single input produces an `order = 0` file that **does** have a `composed_of` (just one entry). That file is semantically composed (has a recipe) but matches an atomic in logical order. Comparing it against a hand-authored atomic of the same order would be ambiguous: would you content-diff (since both are order 0) or structural-diff (since one has a recipe)?

Refusing cleanly is the right call. If you actually want to compare the rendered text of a composed-order-0 against an atomic, decompose the composed one first.

---

## Failure modes

Beyond the dispatch refusals listed above:

| Trigger                                       | Error                                                                                       |
|-----------------------------------------------|---------------------------------------------------------------------------------------------|
| Either file does not exist                    | [`FileNotFound`](./errors.md#filenotfound) (path is reported)                              |
| Either file fails to parse                    | The corresponding parse error ([`InvalidToml`](./errors.md#invalidtoml), [`MissingOpenDelimiter`](./errors.md#missingopendelimiter), etc.) |
| Either file is missing `composed_of` when expected (would only happen if validator was bypassed) | [`OrderRequiresField`](./errors.md#orderrequiresfield)                                     |

---

## See also

- [schema.md](./schema.md) — what fields are diff'd
- [order-and-delimiters.md](./order-and-delimiters.md) — `order` and `body_level` semantics
- [command-compose.md](./command-compose.md) — produces the files Compare diffs
- [errors.md](./errors.md) — refusal error variants in detail
- [demos/04-structural-diff](../demos/04-structural-diff/) — end-to-end demonstration of structural diff cutting through noise
