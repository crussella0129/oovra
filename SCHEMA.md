+++
name = "Oovra File Format Schema"
kind = "atom"
id = "oovra-schema"
version = "0.2.0"
meta = "Canonical schema for Oovra prompt-element files. Read this first if you are an agent authoring Oovra files."
+++

# Oovra File Format Schema (v0.2)

This is the canonical schema for Oovra files. It is written for both humans and AI agents. If you are an agent authoring Oovra files, the rules here are sufficient — you do not need any external context.

If anything in this document conflicts with the Rust source, **the source is wrong** and should be fixed to match this document. This file is the contract.

The long-form reference (with worked examples, joint-invariant tables, and links to demos) lives at [`Documentation/reference/schema.md`](./Documentation/reference/schema.md). SCHEMA.md is the terse machine-facing summary.

---

## What an Oovra File Is

An Oovra file is a single `.md` file with two parts:

1. A **TOML frontmatter block** delimited by `+++` on its own line at the top and `+++` on its own line at the end.
2. A **Markdown body** following the closing `+++`, separated by exactly one blank line.

A file missing either part is rejected.

---

## One Schema, One Discriminator: `kind`

Every Oovra file is a **prompt element**. A required `kind` field tells the parser (and the agent) which of two shapes the file has:

- **`kind = "atom"`** — hand-authored. A self-consistent sentence or paragraph that holds together on its own. No recipe; no composition metadata.
- **`kind = "compound"`** — produced by `oovra compose` from inputs. Carries a `composed_of` recipe plus `generated_at`, `render_mode`, `body_level`, and (optionally) `depth`.

You cannot hand-author a compound: the parser rejects any file with `kind = "compound"` that lacks a `composed_of` recipe, and any file with `kind = "atom"` that carries compound-only fields. Use `oovra compose` to produce compounds.

> **v0.1 migration note.** v0.1 used a numeric `order` field as the discriminator. v0.2 replaces it with explicit `kind`. Run `oovra migrate <library-dir>` to rewrite a v0.1 library in place. The `--legacy` global flag accepts v0.1 files for read-only operations during transition. See [v0.2 SPEC §5](./Documentation/version-reports/v0.1/SPEC-v0.2.md).

---

## Delimiter Rules

The frontmatter delimiter is exactly `+++` on a line by itself — **not** `---`. The choice is intentional: `---` signals YAML; `+++` signals TOML; an Oovra file is unambiguous from its first line.

Strict rules:
- The opening `+++` must be the first line of the file.
- The closing `+++` must be on its own line.
- Exactly one blank line follows the closing `+++`, then the body.
- The body may contain `+++` characters anywhere (they're just text).

---

## Schema for All Prompt Elements

### Required fields (every element — atom or compound)

| Field | TOML type | Constraints |
|---|---|---|
| `name` | string | non-empty; the human-readable display name |
| `kind` | string | exactly `"atom"` or `"compound"` |
| `id` | string | kebab-case (lowercase, digits, hyphens; no leading/trailing/double hyphens); unique across the library |
| `version` | string | valid semver (e.g. `"1.0.0"`, `"2.3.1-rc1"`) |
| `meta` | string | the customizable description; may be empty (`""`) |

### Required when `kind = "compound"`

| Field | TOML type | Constraints |
|---|---|---|
| `generated_at` | string | RFC 3339 timestamp (e.g. `"2026-05-09T14:23:15Z"`) |
| `render_mode` | string | identifies the renderer; v0.2 supports `"markdown-h2"` |
| `body_level` | non-negative integer (≥ 1) | the **physical body delimiter level** — always `max(input.body_level, default = 0) + 1` at compose time |
| `composed_of` | array of inline tables `{id, version}` | the immediate inputs (one level down) in compose order; non-empty |

### Optional on compounds, written by `oovra compose`

| Field | TOML type | Constraints |
|---|---|---|
| `depth` | non-negative integer (≥ 1) | `max(input.depth, default = 0) + 1`; numerically equal to `body_level` for every valid compound |

### Forbidden on atoms

When `kind = "atom"`, **none** of `composed_of`, `generated_at`, `render_mode`, `body_level`, `depth` may be present. Any compound-only field on an atom is an `AtomHasForbiddenField` error.

### Invariants between fields

- An atom has `kind = "atom"`, no `composed_of`, and no compound-only companions. The validator rejects any other shape.
- A compound has `kind = "compound"`, all four required companions, plus an optional `depth`. Its `body_level` always satisfies the strict-escalation rule `max(input.body_level, default = 0) + 1`.

### Two integers, one number: `body_level` and `depth`

These look similar but answer different questions:

- **`body_level`** is the *physical* delimiter level used by the body parser — always `max(input.body_level, default = 0) + 1`. This is what makes the body parser unambiguous (strict tilde-count escalation).
- **`depth`** is the *human-friendly* compositional-depth label — `max(input.depth, default = 0) + 1`. Numerically identical to `body_level` for every valid v0.2 compound; exposed as a separate field so future renderers can change the delimiter convention without renaming the user-visible depth metric.

Both increment from the max input value by exactly one. Both treat atoms as contributing 0.

### Body rules

- **For atoms**: the body is freeform Markdown — the actual prompt text. It must be non-empty after stripping whitespace.
- **For compounds**: the body is a sequence of K complete sub-element files (each with their own frontmatter), wrapped in chiral level-aware delimiters whose tilde count is set by `body_level`. See below.

---

## Body Delimiter Scheme (compounds only)

When `oovra compose` produces a compound, its body is the **concatenation of the full file content of each input** (frontmatter + body of each input), wrapped in chiral delimiters whose tilde count is determined by the parent's `body_level` field.

| `body_level` | Open | Close |
|---|---|---|
| 1 | `~~>>` | `~~<<` |
| 2 | `~~~>>` | `~~~<<` |
| N | `(N+1)` tildes + `>>` | `(N+1)` tildes + `<<` |

The parser reads `body_level` from the element's header and scans for delimiters with exactly that tilde count.

**Properties:**

- **Chiral**: open ≠ close. Mismatched delimiters are a parse error.
- **Strictly monotonic**: a level-N delimiter has *more* tildes than any level less than N. The outer parser, scanning for `(N+1)` tildes, ignores every inner level-(N−k) delimiter.
- **Each chunk is itself a complete Oovra file.** This means `decompose --full` can recursively split bodies and recover every leaf — including all metadata (name, version, meta) — without consulting any external state.

### Example: a body_level = 1 compound body

```
~~>>
+++
name = "Strict Refusal Policy"
kind = "atom"
id = "refusal-policy-strict"
version = "1.0.0"
meta = "..."
+++

(the atom body text)
~~<<
~~>>
+++
name = "Tone: Direct"
kind = "atom"
id = "tone-direct"
version = "1.0.0"
meta = "..."
+++

(the atom body text)
~~<<
```

---

## Validation Rules (What the Parser Checks)

### Lexical and structural (all elements)

1. The file begins with `+++` on its own line.
2. A second `+++` line appears later.
3. The content between is valid TOML.
4. `name`, `kind`, `id`, `version`, `meta` are present with correct types. `kind` must be exactly `"atom"` or `"compound"`.
5. The body is non-empty after stripping whitespace.

### Semantic

1. `id` matches the kebab-case grammar.
2. `version` parses as semver.
3. `name` is non-empty.

### Kind-specific (atoms)

1. None of `composed_of`, `generated_at`, `render_mode`, `body_level`, `depth` may be present.

### Kind-specific (compounds)

1. All of `composed_of` (non-empty), `generated_at` (RFC 3339), `render_mode`, `body_level` (≥ 1) must be present.
2. Every entry of `composed_of` has a kebab-case `id` and a semver `version`.
3. If `depth` is present, it must be ≥ 1.

### Library-wide

1. All `id` values across the library are unique.

---

## Operators

The CLI is `oovra`, with four operators plus a migration helper:

- **`oovra create`** — author a new atom (`--new`) or label an existing `.md` file (`--label`). Always produces atoms.
- **`oovra compose`** — combine ordered inputs into a compound. Computes `body_level` and `depth`. Three modes: writes a file (default), prints prose (`--text`), or regenerates the body of an existing compound (`--re-render <path>`).
- **`oovra decompose`** — split a compound. Default returns the immediate inputs (one level). `--full` recursively writes a folder tree all the way to atom leaves.
- **`oovra compare`** — diff two prompt elements. Atom+atom → content diff. Compound+compound → **sequence-aware structural diff** over `composed_of` (added / removed / version-changed / moved inputs). Atom+compound → refused (`KindMismatch`).
- **`oovra migrate <dir>`** — walk a v0.1 library and rewrite every Oovra file in v0.2 schema, in place. Idempotent on already-v0.2 files.

Validation is internal to every operator. There is no separate `validate` or `inspect` command — running `oovra compose --text <id>` on a single element loads it, validates it, and prints the body without writing anything.

The global `--legacy` flag accepts v0.1-schema files (`order` instead of `kind`) for read operations during the migration window. Writes are always in v0.2 format regardless of `--legacy`.

---

## Guidance for Agents

If you are an LLM authoring Oovra files, follow these rules to maximize first-shot correctness:

- **Always include all five required fields** (`name`, `kind`, `id`, `version`, `meta`). `meta = ""` is fine; missing `meta` is a parse error.
- **Hand-authored files are always `kind = "atom"`.** Do not write `kind = "compound"` by hand. Use `oovra compose`.
- **Quote all string values.** TOML allows bare strings in some contexts but always quoting eliminates a class of edge cases.
- **Use kebab-case for all IDs.** Lowercase letters, digits, hyphens; no leading/trailing/double hyphens.
- **Use semver for all versions.** `"1.0.0"`, not `"v1.0"` or `"1.0"`.
- **Write portable bodies.** Atom bodies should make sense in any composition. Avoid "as I said above" / "the next section" — those break when the element is composed in a different position.
- **When in doubt, run `oovra compose --text <id>`** on the element you just authored. If it parses, you'll see the body. If not, you'll see a clear error.

---

## Quick Reference Card

```toml
+++
name = "Human-Readable Name"
kind = "atom"
id = "kebab-case-id"
version = "1.0.0"
meta = "optional description; may be empty string"
# When kind = "compound", these are also required (and forbidden when kind = "atom"):
# generated_at = "RFC 3339"
# render_mode = "markdown-h2"
# body_level = 1                # = max(input.body_level, default = 0) + 1
# composed_of = [{ id = "...", version = "..." }, ...]
# depth = 1                     # optional; numerically equal to body_level
+++

(body — freeform Markdown for atoms;
 wrapped sub-element files for compounds)
```

If you remember nothing else:

- `+++` not `---`
- `kind = "atom"` for hand-authored, `kind = "compound"` for `oovra compose` output
- IDs are kebab-case
- Versions are semver
- Run `oovra compose --text <id>` to verify
