# `oovra compose` — Reference

Joins prompt elements from a library into a **compound** output. The JOIN operator. Has three modes selected by flag combinations.

> **v0.2 migration note.** This page documents the v0.2 schema. The `order` integer is gone; compound outputs are tagged `kind = "compound"` and carry a `depth` field next to the existing `body_level`. If you have v0.1 files on disk, run `oovra migrate <library-dir>` to convert them in place.

## Synopsis

```
oovra compose <ID>...                              [common flags]   # File mode: write a new composed .md
oovra compose <ID>... --text                       [common flags]   # Text mode: print clean prose to stdout
oovra compose --re-render <PATH>                   [common flags]   # Re-render an existing compound file's body
```

Common flags across all modes:

| Flag                  | Default          | Purpose                                                  |
|-----------------------|------------------|----------------------------------------------------------|
| `--library <DIR>`     | `./elements`     | Directory to resolve input IDs from (recursive walk)     |
| `--out-id <ID>`       | `composed-<IDs-joined>` | ID for the produced compound file                   |
| `--out-name <NAME>`   | the value of `--out-id` | Human-readable name for the produced file           |
| `--out-version <V>`   | `"1.0.0"`        | Semver string for the produced file                      |
| `--out-meta <META>`   | `""`             | Free-form description                                    |

---

## Mode 1: file (default) — write a compound .md

```
oovra compose <ID-1> <ID-2> ... <ID-N>  --output <PATH>  [common flags]
```

Resolves each `<ID>` against the library, computes the output's `body_level` and `depth`, renders the body (embedding each input's full source file wrapped in chiral delimiters), and writes a complete Oovra file to `--output <PATH>`.

### Flags specific to this mode

| Flag                   | Required?      | Default              | Purpose                                                                                                                    |
|------------------------|----------------|----------------------|----------------------------------------------------------------------------------------------------------------------------|
| `<ID>...` positional   | yes (≥1)       | —                    | Input element IDs in the sequence they should appear in the composition. The sequence is preserved in `composed_of` and the body. |
| `-o, --output <PATH>`  | no             | `<library>/<out-id>.md` | Where to write the compound file. Defaults to your library directory + the out-id + `.md`.                              |

### What's produced

A complete Oovra file at the output path. Its frontmatter has:

- `name` ← `--out-name` (or `--out-id` if not given)
- `kind` ← `"compound"` (always — Mode 1 only produces compounds)
- `id` ← `--out-id` (or auto-generated)
- `version` ← `--out-version` (default `"1.0.0"`)
- `meta` ← `--out-meta` (default `""`)
- `generated_at` ← current UTC time as RFC 3339 (set by the tool, never user-provided)
- `render_mode` ← `"markdown-h2"` (v0.2 has one renderer)
- `body_level` ← computed by `compute_body_level(input_body_levels) = max(input.body_level, default = 0) + 1`
- `depth` ← computed by `compute_depth(input_depths) = max(input.depth, default = 0) + 1` (numerically equal to `body_level` for every valid compound; see [kind-and-delimiters.md](./kind-and-delimiters.md))
- `composed_of` ← array of `{id, version}` for each input, in argument order, with versions from the library

The body is the concatenation of each input's full file content (frontmatter + body), each wrapped between `~~...~~>>` and `~~...~~<<` lines whose tilde count is `body_level + 1`. See [kind-and-delimiters.md](./kind-and-delimiters.md#the-delimiter-scheme) for the full spec.

### Example: 3 atoms → a body_level-1 compound

```bash
oovra compose --library ./elements \
  --out-id coding-agent --out-name "Coding Agent" --out-meta "Strict 3-element pair" \
  -o ./elements/coding-agent.md \
  role-statement safety-fence tone-direct
```

Output (header excerpt):

```toml
+++
name = "Coding Agent"
kind = "compound"
id = "coding-agent"
version = "1.0.0"
meta = "Strict 3-element pair"
generated_at = "2026-05-10T18:56:15.993212217+00:00"
render_mode = "markdown-h2"
body_level = 1
depth = 1

[[composed_of]]
id = "role-statement"
version = "1.0.0"

[[composed_of]]
id = "safety-fence"
version = "1.0.0"

[[composed_of]]
id = "tone-direct"
version = "1.0.0"
+++
```

Followed by a body containing three `~~>> ... ~~<<` chunks — one per input, each containing that input's full file content.

### Ordering matters

Inputs are emitted in the body in the **exact sequence** of the positional arguments. The `composed_of` array preserves this sequence. This affects:

- The sequence H2 sections appear in `compose --text` output
- The sequence leaves appear in `decompose --full` output (when listed)
- `compare`'s structural diff (see notes there about set vs sequence semantics)

If you want a different rendering sequence, re-run Compose with a different argument order.

### Failure modes

| Trigger                                                       | Error                                                                                          |
|---------------------------------------------------------------|------------------------------------------------------------------------------------------------|
| Zero positional input IDs (without `--re-render`)             | [`EmptyCompose`](./errors.md#emptycompose)                                                     |
| `<ID>` not found in library                                   | [`ElementNotFound`](./errors.md#elementnotfound)                                               |
| Library directory has duplicate IDs                           | [`DuplicateId`](./errors.md#duplicateid) (raised by `Library::load` before compose runs)        |
| Output `--out-id` is not kebab-case                           | [`InvalidField { field: "id" }`](./errors.md#invalidfield) — file is **not** written           |
| `--output` path is in a directory that doesn't exist          | The directory is auto-created; no error in normal cases. [`WriteIo`](./errors.md#io--writeio) if creation fails (permissions). |

---

## Mode 2: `--text` — print clean prose to stdout

```
oovra compose <ID-1> <ID-2> ... <ID-N>  --text  [common flags]
```

Resolves the inputs, recursively flattens through every level of composition, and prints clean Markdown to stdout. **Nothing is written to disk.** The output is intentionally **not** a valid Oovra file — there's no frontmatter, no delimiters, just `## <id>` headers and the bodies.

### What you get

For each atom in the tree, one section:

```markdown
## <id-of-leaf-1>

<body of leaf 1>

## <id-of-leaf-2>

<body of leaf 2>

...
```

Compound inputs are **recursed through**: their own ID never appears in the output, only the IDs of the atom leaves they ultimately contain. A `depth = 3` input flattens to whatever atom leaves are inside it, in composition order.

This mode is the intended way to get a paste-ready system prompt out of Oovra. It is **lossy** — version numbers, metadata, generation timestamps don't appear — but that's intentional. A model doesn't need that information; you do, and you can recover it from the on-disk form.

### Example

Library has `role-statement` (atom), `safety-fence` (atom), `tone-direct` (atom).

```bash
oovra compose --library ./elements --text role-statement safety-fence tone-direct
```

Output to stdout:

```markdown
## role-statement

You are a senior software engineer pairing with the developer at the keyboard. Stay grounded in the actual code at hand.

## safety-fence

When asked to produce code that would damage a system, decline once briefly and offer the closest legitimate alternative.

## tone-direct

Be direct. Skip preamble. Skip apology. State conclusions before reasoning.
```

### `--text` with compound inputs

If one of the input IDs is itself a compound element, the recursion descends into it. For example, if `coding-agent` is a compound (body_level 1) composed of three atoms, then:

```bash
oovra compose --library ./elements --text coding-agent
```

produces the same three H2 sections as if you had typed the three atom IDs directly. The compound wrapper is invisible in the prose form. See [demos/03-deep-text-flattening](../demos/03-deep-text-flattening/) for a body_level-2 example.

### Failure modes

| Trigger                          | Error                                                |
|----------------------------------|------------------------------------------------------|
| Zero positional IDs              | clap error (positional required)                     |
| `<ID>` not in library            | `anyhow!("element '{id}' not found in library")`     |
| Sub-element decompose fails      | [`BodyParse`](./errors.md#bodyparse) (rare; would indicate a corrupted compound file in the library) |

`--text` cannot conflict with `--output` (clap-level conflict). Producing a file *and* printing prose are different operations — choose one.

---

## Mode 3: `--re-render <PATH>` — regenerate an existing compound file's body

```
oovra compose --re-render <PATH>  [common flags except positional IDs]
```

Reads the existing compound file at `<PATH>`, extracts its `composed_of` recipe, re-resolves every entry against the current library state with **strict version-pin enforcement**, and overwrites the file's body with a freshly rendered version.

### Use case

You've changed something about a renderer or moved code around and want the body bytes regenerated without changing the recipe or the version pins. Useful for:

- Refreshing `generated_at` to track when the file was last rebuilt
- Catching cases where the body has been hand-edited (the re-render will diverge from the hand-edit, surfacing the drift)
- Future-proofing for when a future renderer ships — re-render the whole library against the new renderer

### Strict pin enforcement

Unlike Mode 1 (which picks up whatever version is in the library), Mode 3 enforces that each input's library version **exactly matches** the version pinned in the file's existing `composed_of`. If any input's version has changed in the library, re-render fails with [`VersionMismatch`](./errors.md#versionmismatch).

This is intentional: if you wanted to absorb new versions, you'd run Mode 1 with the same input IDs (no pin enforcement). Re-render is for "regenerate against the SAME inputs at the SAME versions" — drift detection, not drift absorption.

### Example

Existing `./elements/coding-agent.md` has `composed_of` pinning `role-statement = 1.0.0`, `safety-fence = 1.0.0`, `tone-direct = 1.0.0`.

If the library still has those versions:

```bash
oovra compose --library ./elements --re-render ./elements/coding-agent.md
# → Re-rendered ./elements/coding-agent.md (body_level 1)
```

If `role-statement` has been bumped to `1.1.0` in the library:

```bash
oovra compose --library ./elements --re-render ./elements/coding-agent.md
# → Error: Version mismatch for 'role-statement': pin '1.0.0' does not match library version '1.1.0'
```

You'd then either revert the version, or re-run Mode 1 to produce a new composition with the new version:

```bash
oovra compose --library ./elements --out-id coding-agent --out-version 1.1.0 \
  -o ./elements/coding-agent.md \
  role-statement safety-fence tone-direct
```

### Failure modes

| Trigger                                                       | Error                                                                                          |
|---------------------------------------------------------------|------------------------------------------------------------------------------------------------|
| `<PATH>` does not exist                                       | [`FileNotFound`](./errors.md#filenotfound)                                                     |
| `<PATH>` is an atom (no `composed_of`)                        | `anyhow!("--re-render target is an atom and has no composed_of")`                              |
| Library is missing an input named in `composed_of`            | [`ElementNotFound`](./errors.md#elementnotfound)                                               |
| Library version of an input differs from its pin              | [`VersionMismatch`](./errors.md#versionmismatch)                                               |

---

## Output sizing: how `body_level` and `depth` are determined

When Compose produces a file, it computes two numbers from the inputs. Both use the same strict-escalation rule — the v0.1 count-based `compute_order` formula is gone.

**`body_level`** (the physical on-disk delimiter level):

```text
output.body_level = max(input.body_level, default = 0) + 1
```

Atoms contribute `body_level = 0` (they have no `body_level` field on disk; the formula treats absence as 0).

**`depth`** (the human-friendly compositional-depth label, mirrors `body_level`):

```text
output.depth = max(input.depth, default = 0) + 1
```

The two are numerically equal for every valid compound; `depth` is exposed as a separate field so downstream tooling can read "how deep is this recipe tree" without knowing the delimiter-level convention. See [kind-and-delimiters.md](./kind-and-delimiters.md).

### Worked examples

| Inputs                                              | `body_level` | `depth` |
|-----------------------------------------------------|--------------|---------|
| 3 × atom                                            | 1            | 1       |
| 2 × compound at body_level 1                        | 2            | 2       |
| 1 × compound at body_level 1 + 2 × atom             | 2            | 2       |
| 1 × compound at body_level 2 + 1 × compound at body_level 1 + 4 × atom | 3            | 3       |

---

## See also

- [schema.md](./schema.md) — the file format Compose produces
- [kind-and-delimiters.md](./kind-and-delimiters.md) — the formulas and the delimiter spec
- [command-decompose.md](./command-decompose.md) — the inverse operation; reads what Compose produces
- [errors.md](./errors.md) — every error Compose can surface
- [demos/03-deep-text-flattening](../demos/03-deep-text-flattening/) — `--text` on a body_level-2 compound
- [demos/05-mixed-order-regression](../demos/05-mixed-order-regression/) — the mixed-input case the strict-escalation rule was designed for
