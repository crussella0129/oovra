# Schema Reference

Every Oovra file follows one of two shapes, governed by a single required field: `kind`. Files with `kind = "atom"` are hand-authored (or produced by `oovra create`); files with `kind = "compound"` are produced by `oovra compose` and carry a `composed_of` recipe.

This document is the canonical reference for what fields exist, what types they have, what constraints they must satisfy, and what invariants the parser enforces.

> **Vocabulary** — every Oovra file is a *prompt element*. The two **kinds** of prompt element are **atom** and **compound**. "Element" is the umbrella; "atom" and "compound" are the two shapes. See [v0.2 SPEC §1.5](../version-reports/v0.1/SPEC-v0.2.md) for the rationale.

---

## File anatomy

```
+--------------------------+
|        +++               | <- opening delimiter (line 1 of file, exactly three plus signs)
|                          |
|  <TOML frontmatter>      | <- the header: 5 required fields + kind-specific fields
|                          |
|        +++               | <- closing delimiter
|                          | <- exactly ONE blank line
|                          |
|  <Markdown body>         | <- atom: freeform prose; compound: wrapped sub-element files
|                          |
+--------------------------+
```

**Strict rules for the delimiter pair:**

- The opening `+++` must be **line 1** of the file. Not a blank line. Not a comment.
- The opening `+++` must consist of *exactly* three plus signs followed by end-of-line. No leading whitespace, no trailing characters.
- The closing `+++` must be on its own line, with the same constraints.
- Between the closing `+++` and the body must be **exactly one** blank line. Multiple blank lines after the closing delimiter are part of the body.
- The body may contain `+++` lines anywhere; the parser only cares about the first two.

---

## The five always-required fields

Every Oovra file — atom or compound — must have these five fields. The validator rejects any file missing any of them.

| Field        | TOML type | Constraint                                                                  | Purpose                                                                                                      |
|--------------|-----------|-----------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------|
| `name`       | string    | non-empty after trim                                                        | Human-readable display name; what tools show in lists and CLI output                                         |
| `kind`       | string    | exactly `"atom"` or `"compound"`                                            | The canonical discriminator. Determines which set of conditional fields applies.                             |
| `id`         | string    | kebab-case (lowercase ASCII letters, digits, `-`; no leading/trailing/double hyphens); unique within a library | Stable identifier used by `composed_of` references and CLI commands                                          |
| `version`    | string    | valid [semver](https://semver.org/) (e.g. `"1.0.0"`, `"2.3.1-rc1"`)         | Version pinning across the library; bump on every meaningful body change                                     |
| `meta`       | string    | may be empty (`""`)                                                         | Free-form description; not consumed by Oovra itself, but shown in tools                                      |

### `id` — the kebab-case grammar

```
id  ::= word ('-' word)*
word ::= [a-z0-9]+
```

In words: lowercase ASCII letters and digits, with hyphens between word segments. No leading or trailing hyphen. No double hyphens. No underscores. No uppercase.

Valid: `role-statement`, `policy-007`, `v2-beta`, `a`, `a-b-c`
Invalid: `RoleStatement` (capitals), `role_statement` (underscore), `role statement` (space), `-leading`, `trailing-`, `double--dash`, `""` (empty)

### `version` — semver constraints

Three numeric components separated by dots, optionally followed by a pre-release suffix and/or build metadata. This is the standard [semver 2.0](https://semver.org/) grammar.

Valid: `"1.0.0"`, `"0.1.0"`, `"2.3.1-rc1"`, `"1.0.0-alpha.1+build.42"`
Invalid: `"v1.0"` (leading `v`), `"1.0"` (only two components), `"1"` (only one component), `""` (empty)

### `meta` vs missing-`meta`

`meta` is **always required** as a field but **may be the empty string**. This keeps the schema rectangular — `meta = ""` is valid; *omitting* the field is a parse error. Rectangular schemas are cheaper to validate and round-trip.

---

## Fields specific to compounds

When `kind = "compound"`, these five additional fields apply. The first four are required; `depth` is optional.

| Field           | TOML type                                                | Constraint                                                            | Purpose                                                                                                |
|-----------------|----------------------------------------------------------|-----------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| `generated_at`  | string                                                   | RFC 3339 timestamp with timezone (e.g. `"2026-05-10T18:56:15Z"`)      | When Compose produced this file; useful for reproducibility tracking                                   |
| `render_mode`   | string                                                   | identifies which renderer wrote the body; v0.2 supports `"markdown-h2"` | Future-proofing for alternate renderers; current value tells decompose which delimiter scheme to expect |
| `body_level`    | non-negative integer                                     | `≥ 1`; equals `max(input.body_level, default = 0) + 1` at compose time | Physical body-delimiter level; determines how many tildes the parser looks for                          |
| `composed_of`   | array of inline tables `{ id = "...", version = "..." }` | non-empty; every entry's `id` is kebab-case, every `version` is semver | The recipe: ordered list of immediate inputs (one level down)                                          |
| `depth`         | non-negative integer (optional)                          | `≥ 1`; equals `1 + max(child.depth, atoms = 0)`; numerically equal to `body_level` for any valid compound | Human-readable compositional depth; exists for filtering and tooling                                   |

When `kind = "atom"`, **none** of these fields may be present. Setting any of them on an atom is an [`AtomHasForbiddenField`](./errors.md#atomhasforbiddenfield) error.

### `composed_of` syntax in TOML

`composed_of` can be written two equivalent ways in TOML.

**Inline-table form** (compact, used by Compose's output):

```toml
composed_of = [
  { id = "role-statement", version = "1.0.0" },
  { id = "safety-fence", version = "1.0.0" }
]
```

**Array-of-tables form** (readable for many entries):

```toml
[[composed_of]]
id = "role-statement"
version = "1.0.0"

[[composed_of]]
id = "safety-fence"
version = "1.0.0"
```

Both parse to the same `Vec<InputRef>`. Either form is accepted on input; Compose currently emits the inline form.

### `depth` vs `body_level`

For any valid compound, `depth` and `body_level` are numerically equal. They differ in audience:

- `body_level` is what the **body parser** scans for — it directly determines the tilde count of the chiral delimiters around each sub-element.
- `depth` is for **humans and tools**: a single integer answering "how deeply composed is this?" without anyone needing to remember the parser's delimiter scheme.

Compose always writes both. The validator checks `body_level` strictly and checks `depth ≥ 1` when present.

---

## Field-presence invariants (the joint validator)

The validator dispatches on `kind`. There are exactly two valid presence-states:

| State        | `kind`       | `composed_of` | `generated_at` | `render_mode` | `body_level` | `depth`     | Description                                                          |
|--------------|--------------|---------------|----------------|---------------|--------------|-------------|----------------------------------------------------------------------|
| **Atom**     | `"atom"`     | absent        | absent         | absent        | absent       | absent      | Hand-authored or `oovra create`-produced; body is freeform prose     |
| **Compound** | `"compound"` | present       | present        | present       | present      | optional    | `oovra compose`-produced; body is wrapped sub-element files          |

Any other combination is rejected at parse time:

- Atom with any compound-only field → [`AtomHasForbiddenField`](./errors.md#atomhasforbiddenfield) error naming the offending field
- Compound missing any of `composed_of` / `generated_at` / `render_mode` / `body_level` → [`CompoundMissingField`](./errors.md#compoundmissingfield) error naming the missing field
- `kind` field absent → [`InvalidToml`](./errors.md#invalidtoml) ("missing field `kind`")
- `kind` value other than `"atom"` or `"compound"` → [`InvalidToml`](./errors.md#invalidtoml) ("unknown variant `x`, expected `atom` or `compound`")

This invariant is what lets the rest of the codebase rely on `is_atom()` / `is_compound()` predicates without having to check each field individually.

---

## Body content rules

### Atoms

The body is freeform Markdown. There is one constraint: the body must be **non-empty after stripping whitespace** ([`EmptyBody`](./errors.md#emptybody) error otherwise). The body holds the actual policy/prompt text — what this element "says" when included in a composition.

Authoring guidance for atom bodies:

- **Internally consistent** — express one coherent rule that makes sense in isolation
- **Portable** — should read correctly when included in *any* composition; avoid "as I said above" / "the next section" wording
- **Right-sized** — typically one to five sentences. Single sentences are usually too small; multi-paragraph bodies are usually too big

The body may contain Obsidian-style wiki-links (`[[other-element-id]]`); Oovra ignores them but Obsidian renders them as live navigation. They are purely cosmetic from the parser's perspective.

### Compounds

The body is **machine-generated** by Compose. It is **not** intended for hand-editing — if you edit a compound body, the next `compose --re-render` will overwrite your changes.

Each immediate input is embedded as a complete sub-file (frontmatter + body), wrapped in chiral `body_level`-aware delimiters whose tilde count is governed by the `body_level` field. The full delimiter scheme is documented in [kind-and-delimiters.md](./kind-and-delimiters.md).

The body must be non-empty (same rule as atoms). For compounds this is automatic — every composition has at least one input — but it remains a validator check.

---

## Validation rules summary

Applied in this order, with the first failure stopping the parse:

### Lexical and structural (every file)

1. Opening `+++` on line 1.
2. Closing `+++` somewhere later, on its own line.
3. Content between is valid TOML.
4. All five required fields (`name`, `kind`, `id`, `version`, `meta`) are present with correct types. `kind` must be exactly `"atom"` or `"compound"`.

### Semantic (every file)

5. `id` matches the kebab-case grammar.
6. `version` parses as semver.
7. `name` is non-empty after trim.
8. Body is non-empty after trim.

### Kind-specific (atoms)

9. None of `composed_of`, `generated_at`, `render_mode`, `body_level`, `depth` may be present.

### Kind-specific (compounds)

10. All of `composed_of` (non-empty), `generated_at` (RFC 3339), `render_mode`, `body_level` (`≥ 1`) must be present. Every entry of `composed_of` has kebab-case `id` and semver `version`.
11. If `depth` is present, it must be `≥ 1`. (The full equation `depth = 1 + max(child.depth)` is verified by recursive walks, not the per-file validator.)

### Library-wide (when loading via `Library::load`)

12. All `id` values across the library are unique. Duplicates emit a [`DuplicateId`](./errors.md#duplicateid) error pointing at both files.

---

## Example: a complete atom

```toml
+++
name = "Strict Refusal Policy"
kind = "atom"
id = "refusal-policy-strict"
version = "1.2.0"
meta = "Brief, non-preachy decline of harmful requests"
+++

When asked to produce content that would cause concrete harm — including
weapons synthesis, malware, or content sexualizing minors — decline
briefly and without lecturing. One clear sentence of decline is enough.
If a constructive alternative exists, offer it; otherwise, stop.
```

## Example: a complete compound (truncated body)

```toml
+++
name = "Coding Agent Prompt"
kind = "compound"
id = "coding-agent"
version = "1.0.0"
meta = "Three-element strict coding agent"
generated_at = "2026-05-10T18:56:15.993212217+00:00"
render_mode = "markdown-h2"
body_level = 1
depth = 1
composed_of = [
  { id = "role-statement", version = "1.0.0" },
  { id = "refusal-policy-strict", version = "1.2.0" },
  { id = "tone-direct", version = "1.1.0" },
]
+++

~~>>
+++
name = "Role Statement"
kind = "atom"
id = "role-statement"
version = "1.0.0"
meta = "Who the assistant is"
+++

You are a senior software engineer pairing with the developer at the keyboard. ...
~~<<
~~>>
+++
name = "Strict Refusal Policy"
kind = "atom"
...
+++

When asked to produce content that would cause concrete harm — ...
~~<<
~~>>
+++
name = "Direct Tone"
kind = "atom"
...
+++

Be direct. Skip preamble. Skip apology. ...
~~<<
```

Each `~~>> ... ~~<<` pair wraps one full embedded sub-element file (frontmatter + body). The parser scans for those delimiters when `decompose` is called; the embedded files are themselves valid Oovra files and can be re-parsed recursively.

---

## Migrating from v0.1

v0.1 used a numeric `order` field as the discriminator (`order = 0` for atoms, `order ≥ 1` for compounds). v0.2 replaces it with the explicit `kind = "atom" | "compound"` field. Existing v0.1 libraries can be migrated in place:

```sh
oovra migrate <library-dir>
```

The migration is recursive into compound bodies (embedded frontmatters are rewritten too) and idempotent on already-v0.2 files. Run it in a clean Git working tree so the diff is auditable. The `--legacy` global flag accepts v0.1 files for read-only operations during the transition. See [v0.2 SPEC §5](../version-reports/v0.1/SPEC-v0.2.md) for the full mapping rules.

---

## Why the schema looks this way

- **`+++` not `---`** — disambiguates TOML frontmatter from YAML frontmatter without parser sniffing. A reader's tool can pick the right parser from line 1.
- **One struct, two kinds** — `PromptElement` (the umbrella) plus a `kind` discriminator means one parser, one validator, one set of error messages. Operators dispatch on `is_atom()` / `is_compound()` rather than enum-matching the whole struct shape.
- **`kind` not `order`** — the v0.1 numeric `order` field was path-dependent (the same atoms composed via different bracketings produced different `order` values) and carried no information any operator consumed. `kind` is a property of the artifact itself, not of how it was built.
- **`depth` and `body_level` coexist** — `body_level` is what the body parser scans for (tilde count); `depth` is a human-readable equivalent that doesn't require knowing the parser's delimiter scheme. They are numerically identical for every valid compound.
- **Compound bodies are self-describing** — embedding full sub-files (not just IDs) means a compound alone is sufficient to reconstruct every leaf. No library required. See [demos/v0.1/01-lossless-roundtrip](../demos/v0.1/01-lossless-roundtrip/) for the demonstration.

---

## See also

- [kind-and-delimiters.md](./kind-and-delimiters.md) — the chiral delimiter scheme and how `body_level` is computed
- [command-create.md](./command-create.md) — how to produce atoms
- [command-compose.md](./command-compose.md) — how to produce compounds (and the structure of what's produced)
- [errors.md](./errors.md) — every validation error with example triggers
- [demos/v0.1/01-lossless-roundtrip](../demos/v0.1/01-lossless-roundtrip/) — end-to-end proof that the body-as-source-of-truth design holds
- [demos/v0.2/01-operator-gamut](../demos/v0.2/01-operator-gamut/) — full Create→Compose→Decompose→Compare run with the new sequence-aware diff axes
- [v0.2 SPEC](../version-reports/v0.1/SPEC-v0.2.md) — the migration specification
