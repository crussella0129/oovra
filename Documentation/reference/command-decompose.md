# `oovra decompose` — Reference

Recovers the inputs of a compound element. The SPLIT operator. Two modes: a one-level inspection (default) and a recursive full-tree extraction (`--full`).

> **v0.2 migration note.** This page documents the v0.2 schema. The `order` integer is gone; the JSON report now exposes `element_kind` (`"atom"` or `"compound"`) plus `body_level`, and each input entry carries `kind` instead of `order`. If you have v0.1 files on disk, run `oovra migrate <library-dir>` to convert them in place.

## Synopsis

```
oovra decompose <PATH>                                # default: one-level report
oovra decompose <PATH> --format json                  # same, JSON output for tools
oovra decompose <PATH> --full -o <OUTPUT_DIR>         # recursive write to disk
```

## What it does

Reads the file at `<PATH>` and splits its body at the level-`body_level` delimiter pair (`(body_level+1)` tildes + `>>` open, + `<<` close). Each chunk between matched delimiters is itself a complete Oovra file (frontmatter + body). What happens next depends on the mode.

**Critical property**: decompose **does not consult any library**. The compound file's body alone is sufficient to recover every embedded input. This is the load-bearing claim of Oovra's self-describing design — see [demos/01-lossless-roundtrip](../demos/01-lossless-roundtrip/) for the proof.

---

## Mode 1: one-level inspection (default)

```
oovra decompose <PATH>                      # human-readable
oovra decompose <PATH> --format json        # machine-readable
```

Prints a report of the *immediate* inputs (one level down). Does not recurse, does not write anything.

### Flags

| Flag                  | Default          | Purpose                                                |
|-----------------------|------------------|--------------------------------------------------------|
| `<PATH>` (positional) | required         | Path to a compound Oovra file (must have `composed_of`) |
| `--format <FMT>`      | `human`          | `human` for color terminal output, `json` for tools     |
| `-o, --output <DIR>`  | `.`              | Ignored in this mode (only used with `--full`)         |
| `--full`              | (off)            | Switches to Mode 2 — see below                         |

### `--format human` output

Colored, intended for terminal reading:

```
Decompose pairing-agent (1.0.0) — compound at body_level 1, 4 immediate input(s)
  - role-statement @ 1.0.0 (atom) — Role Statement
  - safety-fence @ 1.0.0 (atom) — Safety Fence
  - tone-discipline @ 1.0.0 (atom) — Tone Discipline
  - format-rules @ 1.0.0 (atom) — Format Rules
```

Fields shown per input:
- ID (cyan)
- Version (dimmed, prefixed with `@`)
- Kind (`atom` or `compound`)
- Human-readable name (only shown if different from the ID)

### `--format json` output

Stable JSON for piping into other tools:

```json
{
  "element_id": "pairing-agent",
  "element_kind": "compound",
  "element_version": "1.0.0",
  "body_level": 1,
  "render_mode": "markdown-h2",
  "inputs": [
    { "id": "role-statement", "version": "1.0.0", "kind": "atom", "name": "Role Statement" },
    { "id": "safety-fence", "version": "1.0.0", "kind": "atom", "name": "Safety Fence" },
    { "id": "tone-discipline", "version": "1.0.0", "kind": "atom", "name": "Tone Discipline" },
    { "id": "format-rules", "version": "1.0.0", "kind": "atom", "name": "Format Rules" }
  ]
}
```

This is suitable for use with `jq`, in scripts, or as input to other agentic tools that want to introspect a compound file's recipe.

### What it does NOT do

- It does **not** recurse — even if an input is itself a compound, you only see that input listed once with its own metadata. To recurse, use `--full`.
- It does **not** write anything to disk.
- It does **not** verify the inputs exist in any library; everything reported comes from the file's own body.

### Failure modes

| Trigger                                       | Error                                                                                       |
|-----------------------------------------------|---------------------------------------------------------------------------------------------|
| `<PATH>` is an atom (no `composed_of`)        | [`CannotDecomposeAtom`](./errors.md#cannotdecomposeatom)                                    |
| `<PATH>` file does not exist                  | [`FileNotFound`](./errors.md#filenotfound)                                                  |
| File has frontmatter but malformed body delimiters | [`BodyParse`](./errors.md#bodyparse) — usually means hand-editing has broken the chunks |
| Embedded sub-element doesn't parse as a valid Oovra file | [`InvalidToml`](./errors.md#invalidtoml) or related parsing error               |

---

## Mode 2: `--full` — recursive extraction to disk

```
oovra decompose <PATH> --full  [-o <OUTPUT_DIR>]
```

Recursively splits the file's body at every level, writing each element to disk as its own `.md` file in a folder tree that mirrors the compositional hierarchy.

### Flags

| Flag                  | Default          | Purpose                                                          |
|-----------------------|------------------|------------------------------------------------------------------|
| `<PATH>` (positional) | required         | The compound file to decompose                                    |
| `--full`              | (required)       | Activates this mode                                              |
| `-o, --output <DIR>`  | `.`              | Where to create the output folder. The folder itself is named after the element's ID. |
| `--format <FMT>`      | (ignored)        | Only used in Mode 1                                              |

### Folder structure produced

Given a compound file with ID `<root-id>`, `decompose --full -o <output-dir>` creates:

```
<output-dir>/
└── <root-id>/                          ← always the element's ID, not the filename
    ├── <root-id>.md                    ← a copy of the parent file
    ├── <atom-input-1>.md               ← atom inputs are flat files at this level
    ├── <atom-input-2>.md
    └── <compound-input-id>/            ← compound inputs get their own subdirectory
        ├── <compound-input-id>.md
        ├── <grandchild-leaf-1>.md
        └── <grandchild-leaf-2>.md
```

Rules:

- The **root element** (`<PATH>`) is copied to `<output-dir>/<root-id>/<root-id>.md` so the tree always has a copy of what you started with.
- Each **atom input** is written as a flat `.md` file in its parent's directory. No subdirectory because there's nothing further to decompose.
- Each **compound input** gets its own subdirectory named after its ID, and the recursion descends into it.

For a compound at `depth = N`, the tree can be up to N+1 directories deep.

### Lossless property

Every leaf file written by `--full` is **byte-identical** to the original input file that was composed in. This includes:

- The TOML frontmatter (all 5 required fields)
- The body (every character of prose)
- Trailing newlines, blank lines, whitespace patterns

The `--full` operation can be thought of as a self-extracting archive: hand someone a compound file at any depth, they can `decompose --full` it, and they recover the entire library that produced it — names, versions, meta descriptions, body prose — at byte precision.

See [demos/01-lossless-roundtrip](../demos/01-lossless-roundtrip/) for an end-to-end SHA256-verified proof.

### Example

Compound file `./elements/coding-agent.md` has `body_level = 1` with 4 atom inputs:

```bash
oovra decompose --full -o ./out ./elements/coding-agent.md
# → Decomposed (full) ./elements/coding-agent.md -> ./out/coding-agent
```

Resulting tree:

```
out/
└── coding-agent/
    ├── coding-agent.md           ← copy of the body_level-1 compound root
    ├── role-statement.md         ← atom leaf 1
    ├── safety-fence.md           ← atom leaf 2
    ├── tone-discipline.md        ← atom leaf 3
    └── format-rules.md           ← atom leaf 4
```

For a `body_level = 2` compound, the tree would have subdirectories for each `body_level = 1` child:

```
out/
└── senior-pair-deep/
    ├── senior-pair-deep.md       ← body_level-2 root
    ├── agent-foundations/        ← body_level-1 sub-prompt
    │   ├── agent-foundations.md
    │   ├── role-statement.md
    │   ├── safety-fence.md
    │   └── tone-discipline.md
    └── output-discipline/        ← body_level-1 sub-prompt
        ├── output-discipline.md
        ├── format-rules.md
        ├── examples-discipline.md
        └── file-citation-rules.md
```

See [demos/03-deep-text-flattening](../demos/03-deep-text-flattening/) for the body_level-2 case in full detail.

### What `--full` doesn't do

- It does **not** verify against a library. The recovered files might be older versions than what's in your current library, or might reference IDs that aren't in your current library. The recovered files are recovered from the *file you gave it*, not from any external source.
- It does **not** delete `<output-dir>` first. If `<output-dir>` already contains a `<root-id>/` directory, files in it are overwritten. Other files in `<output-dir>` are left alone.
- It does **not** report individual file writes — just the final summary line.

### Failure modes

Same as Mode 1, plus:

| Trigger                                                       | Error                                                                       |
|---------------------------------------------------------------|-----------------------------------------------------------------------------|
| `<OUTPUT_DIR>` exists but is not writable                     | [`WriteIo`](./errors.md#io--writeio)                                        |
| Recursion encounters a sub-element with a parse error         | The whole operation aborts; partial files may be on disk. Re-run after fixing the input. |

---

## How decompose finds the delimiter level

The parser reads `body_level` from the file's frontmatter and looks for *exactly that many+1* tildes (it's `body_level + 1` tildes on the line — see [kind-and-delimiters.md](./kind-and-delimiters.md) for the formula). It then scans line-by-line for `~~...>>` and `~~...<<` lines with that exact tilde count.

**Inner delimiters are inert**: a `body_level = 2` compound uses 3-tilde delimiters in its body. Embedded `body_level = 1` chunks inside that body have their own 2-tilde delimiters as content — but the outer parser, looking for 3-tilde, treats the 2-tilde lines as ordinary text. This is what makes recursive decomposition unambiguous regardless of depth. See [kind-and-delimiters.md](./kind-and-delimiters.md) for why.

---

## See also

- [command-compose.md](./command-compose.md) — the inverse operation
- [schema.md](./schema.md) — the file format decompose reads
- [kind-and-delimiters.md](./kind-and-delimiters.md) — the chiral delimiter scheme used to split bodies
- [demos/01-lossless-roundtrip](../demos/01-lossless-roundtrip/) — byte-equality demonstration on a simple case
- [demos/03-deep-text-flattening](../demos/03-deep-text-flattening/) — body_level-2 composition with full structure
- [demos/05-mixed-order-regression](../demos/05-mixed-order-regression/) — decompose succeeding on the previously-broken mixed-input case
