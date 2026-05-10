# Schema Reference

Every Oovra file follows one of two shapes, governed by a single flag: whether the file has a `composed_of` recipe. Files **with** `composed_of` are *composed* (produced by `oovra compose`); files **without** are *atomic* (hand-authored, or produced by `oovra create`).

This document is the canonical reference for what fields exist, what types they have, what constraints they must satisfy, and what invariants the parser enforces.

---

## File anatomy

```
+--------------------------+
|        +++               | <- opening delimiter (line 1 of file, exactly three plus signs)
|                          |
|  <TOML frontmatter>      | <- the header: 5 required fields, 4 conditional fields
|                          |
|        +++               | <- closing delimiter
|                          | <- exactly ONE blank line
|                          |
|  <Markdown body>         | <- atomic: freeform prose; composed: wrapped sub-element files
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

Every Oovra file — atomic or composed — must have these five fields. The validator rejects any file missing any of them.

| Field        | TOML type           | Constraint                                                                  | Purpose                                                                                                      |
|--------------|---------------------|-----------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------|
| `name`       | string              | non-empty after trim                                                        | Human-readable display name; what tools show in lists and CLI output                                         |
| `order`      | non-negative integer | atomic files must be `0`; composed files are tool-computed (see [order-and-delimiters.md](./order-and-delimiters.md)) | The element's logical compositional depth                                                                    |
| `id`         | string              | kebab-case (lowercase ASCII letters, digits, `-`; no leading/trailing/double hyphens); unique within a library | Stable identifier used by `composed_of` references and CLI commands                                          |
| `version`    | string              | valid [semver](https://semver.org/) (e.g. `"1.0.0"`, `"2.3.1-rc1"`)         | Version pinning across the library; bump on every meaningful body change                                     |
| `meta`       | string              | may be empty (`""`)                                                         | Free-form description; not consumed by Oovra itself, but shown in tools                                      |

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

## The four conditional fields (composed elements only)

When a file is **composed** — i.e., `composed_of` is present — these four fields must *all* also be present and valid. When `composed_of` is *absent*, all four must also be absent.

| Field           | TOML type                                      | Constraint                                                            | Purpose                                                                                                |
|-----------------|------------------------------------------------|-----------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| `generated_at`  | string                                         | RFC 3339 timestamp with timezone (e.g. `"2026-05-10T18:56:15Z"`)      | When Compose produced this file; useful for reproducibility tracking                                   |
| `render_mode`   | string                                         | identifies which renderer wrote the body; v0.1 supports `"markdown-h2"` | Future-proofing for alternate renderers (Claude XML, plain text, etc.); current value tells decompose which delimiter scheme to expect |
| `body_level`    | non-negative integer                           | `≥ 1` for composed files; equals `max(input.order) + 1` at compose time | Physical body-delimiter level; determines how many tildes the parser looks for                          |
| `composed_of`   | array of inline tables `{ id = "...", version = "..." }` | non-empty; every entry's `id` is kebab-case, every `version` is semver | The recipe: ordered list of immediate inputs (one level down)                                          |

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

Both parse to the same `Vec<InputRef>`. Compose emits the array-of-tables form; both are accepted on input.

---

## Field-presence invariants (the joint validator)

The validator enforces a **joint invariant** between the four conditional fields. There are exactly two valid presence-states:

| State          | `composed_of` | `generated_at` | `render_mode` | `body_level` | `order` | Description                                                          |
|----------------|---------------|----------------|---------------|--------------|---------|----------------------------------------------------------------------|
| **Atomic**     | absent        | absent         | absent        | absent       | must be `0` | Hand-authored or `oovra create`-produced element; body is freeform prose |
| **Composed**   | present       | present        | present       | present      | any         | `oovra compose`-produced element; body is wrapped sub-element files       |

Any other combination is rejected at parse time:

- Atomic file with `order > 0` → [`HandAuthoredHigherOrder`](./errors.md#hand-authored-higher-order) error
- `composed_of` present but `generated_at` missing → [`OrderRequiresField`](./errors.md#orderrequiresfield) error (similar for `render_mode`, `body_level`)
- `composed_of` absent but `generated_at` or `render_mode` or `body_level` present → [`InvalidField`](./errors.md#invalidfield) error indicating "these fields only valid when composed_of is present"

This invariant is what lets the rest of the codebase rely on `is_atomic()` / `is_composed()` predicates without having to check each field individually.

---

## Body content rules

### Atomic elements

The body is freeform Markdown. There is one constraint: the body must be **non-empty after stripping whitespace** ([`EmptyBody`](./errors.md#emptybody) error otherwise). The body holds the actual policy/prompt text — what this element "says" when included in a composition.

Authoring guidance for atomic bodies:

- **Internally consistent** — express one coherent rule that makes sense in isolation
- **Portable** — should read correctly when included in *any* composition; avoid "as I said above" / "the next section" wording
- **Right-sized** — typically one to five sentences. Single sentences are usually too small; multi-paragraph bodies are usually too big

The body may contain Obsidian-style wiki-links (`[[other-element-id]]`); Oovra ignores them but Obsidian renders them as live navigation. They are purely cosmetic from the parser's perspective.

### Composed elements

The body is **machine-generated** by Compose. It is **not** intended for hand-editing — if you edit a composed body, the next `compose --re-render` will overwrite your changes.

Each immediate input is embedded as a complete sub-file (frontmatter + body), wrapped in chiral order-aware delimiters whose tilde count is governed by `body_level`. The full delimiter scheme is documented in [order-and-delimiters.md](./order-and-delimiters.md).

The body must be non-empty (same rule as atomic). For composed elements this is automatic — every composition has at least one input — but it remains a validator check.

---

## Validation rules summary

Applied in this order, with the first failure stopping the parse:

### Lexical and structural (every file)

1. Opening `+++` on line 1.
2. Closing `+++` somewhere later, on its own line.
3. Content between is valid TOML.
4. All five required fields (`name`, `order`, `id`, `version`, `meta`) are present with correct types.

### Semantic (every file)

5. `id` matches the kebab-case grammar.
6. `version` parses as semver.
7. `name` is non-empty after trim.
8. Body is non-empty after trim.

### Joint invariant (every file)

9. If `composed_of` is `Some`: all of `generated_at` (RFC 3339), `render_mode`, `body_level` must also be `Some`. `composed_of` array must be non-empty. Every entry has kebab-case `id` and semver `version`.
10. If `composed_of` is `None`: `order` must be `0`, AND none of `generated_at`/`render_mode`/`body_level` may be set.

### Library-wide (when loading via `Library::load`)

11. All `id` values across the library are unique. Duplicates emit a [`DuplicateId`](./errors.md#duplicateid) error pointing at both files.

---

## Example: a complete atomic file

```toml
+++
name = "Strict Refusal Policy"
order = 0
id = "refusal-policy-strict"
version = "1.2.0"
meta = "Brief, non-preachy decline of harmful requests"
+++

When asked to produce content that would cause concrete harm — including
weapons synthesis, malware, or content sexualizing minors — decline
briefly and without lecturing. One clear sentence of decline is enough.
If a constructive alternative exists, offer it; otherwise, stop.
```

## Example: a complete composed file (truncated body)

```toml
+++
name = "Coding Agent Prompt"
order = 1
id = "coding-agent"
version = "1.0.0"
meta = "Three-element strict coding agent"
generated_at = "2026-05-10T18:56:15.993212217+00:00"
render_mode = "markdown-h2"
body_level = 1

[[composed_of]]
id = "role-statement"
version = "1.0.0"

[[composed_of]]
id = "refusal-policy-strict"
version = "1.2.0"

[[composed_of]]
id = "tone-direct"
version = "1.1.0"
+++

~~>>
+++
name = "Role Statement"
order = 0
id = "role-statement"
version = "1.0.0"
meta = "Who the assistant is"
+++

You are a senior software engineer pairing with the developer at the keyboard. ...
~~<<
~~>>
+++
name = "Strict Refusal Policy"
...
+++

When asked to produce content that would cause concrete harm — ...
~~<<
~~>>
+++
name = "Direct Tone"
...
+++

Be direct. Skip preamble. Skip apology. ...
~~<<
```

Each `~~>> ... ~~<<` pair wraps one full embedded sub-element file (frontmatter + body). The parser scans for those delimiters when `decompose` is called; the embedded files are themselves valid Oovra files and can be re-parsed recursively.

---

## Why the schema looks this way

- **`+++` not `---`** — disambiguates TOML frontmatter from YAML frontmatter without parser sniffing. A reader's tool can pick the right parser from line 1.
- **One schema, not two** — collapsing the original `kind = "node" | "completed"` discriminator into a single struct (gated by `composed_of` presence) means one parser, one validator, one set of error messages. Operators dispatch on `is_atomic()` / `is_composed()` rather than enum-matching.
- **`order` separate from `body_level`** — they answer different questions. `order` is the user-facing logical depth (per the count-based formula). `body_level` is the on-disk delimiter level (always strictly escalating). Decoupling them lets the user's formula stay semantic while the parser stays unambiguous. See [order-and-delimiters.md](./order-and-delimiters.md).
- **Composed bodies are self-describing** — embedding full sub-files (not just IDs) means a composed file alone is sufficient to reconstruct every leaf. No library required. See [demos/01-lossless-roundtrip](../demos/01-lossless-roundtrip/) for the demonstration.

---

## See also

- [order-and-delimiters.md](./order-and-delimiters.md) — the chiral delimiter scheme and how `body_level` is computed
- [command-create.md](./command-create.md) — how to produce atomic elements
- [command-compose.md](./command-compose.md) — how to produce composed elements (and the structure of what's produced)
- [errors.md](./errors.md) — every validation error with example triggers
- [demos/01-lossless-roundtrip](../demos/01-lossless-roundtrip/) — end-to-end proof that the body-as-source-of-truth design holds
