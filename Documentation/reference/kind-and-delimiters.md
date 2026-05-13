# Kind and Delimiters — Specification

Oovra stores **one** integer that governs the on-disk body structure: `body_level`. A second integer, `depth`, is also written by `compose` but is numerically identical to `body_level` and exists for human-readable reporting only — it is not consulted by any parser.

This page is the full specification of `body_level`, the rules for computing it, and the chiral delimiter scheme it governs.

> **Migration note (v0.2)** — v0.1 had two integers (`order` and `body_level`) that diverged in some mixed-kind compositions, leading to a real parser collision bug. v0.2 removes `order`; `body_level` alone now governs both the parser scheme and the user-facing depth. See [v0.2 SPEC §1](../version-reports/v0.1/SPEC-v0.2.md) for the rationale.

---

## `body_level` and `depth`

| Aspect          | `body_level`                                                                       | `depth`                                                                          |
|-----------------|------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| Stored on       | every compound (required); absent on atoms                                         | every compound (optional, always written by `compose`); absent on atoms          |
| Source of value | `max(input.body_level, atoms = 0) + 1`                                             | `max(input.depth, atoms = 0) + 1` — numerically identical to `body_level`        |
| Range           | integer `≥ 1`                                                                      | integer `≥ 1` when present                                                       |
| Audience        | the body parser (determines tilde count of chiral delimiters)                      | humans and tooling (filtering, sorting, "how deep is this?" queries)             |
| Validator       | strictly required and checked on every compound                                    | checked only when present (`depth ≥ 1`); full equation check deferred to library walks |

The two fields exist for different audiences. The parser scans tildes — `body_level` tells it how many. A human reading the frontmatter wants a single integer answering "compositional depth," ideally without knowing the parser's delimiter scheme. `depth` is that integer.

---

## Computing `body_level`

The rule is straightforward:

```
output.body_level = max(input.body_level, default = 0) + 1
```

Atoms contribute `body_level = 0` (their `body_level` field is absent by convention). Compounds contribute their stored `body_level`. The output is always `max + 1`, guaranteeing strict monotonicity of the chiral-delimiter scheme.

### Worked examples

| Inputs                                              | Max input body_level | Output `body_level` |
|-----------------------------------------------------|----------------------|---------------------|
| 3 atoms                                             | 0                    | 1                   |
| 2 compounds, both at `body_level = 1`              | 1                    | 2                   |
| 1 atom                                              | 0                    | 1                   |
| 1 compound (`body_level = 1`) + 4 atoms             | 1                    | 2                   |
| 1 compound (`body_level = 2`) + 1 compound (`bl=1`) + 5 atoms | 2          | 3                   |
| 2 compounds at `body_level = 2` + 1 at `bl = 1`     | 2                    | 3                   |
| 3 compounds at `bl = 1` + 2 atoms                   | 1                    | 2                   |

### Why `body_level` must always escalate

The body of a compound embeds each input as a **complete sub-file** (frontmatter + body), wrapped in delimiters. If an input is itself a compound, its body contains delimiters from *its own* `body_level`. For the outer parser to unambiguously find its delimiters and not accidentally match the inner ones, the outer delimiter must be **strictly different** (specifically, *more tildes*) than any inner one.

Since the deepest inner delimiter has tilde count `max(input.body_level) + 1`, the outer level must be `> max(input.body_level)`. Adding 1 is the minimum-bump that guarantees no collision.

---

## Computing `depth`

```
output.depth = max(input.depth, default = 0) + 1
```

This mirrors `body_level` exactly. Atoms contribute 0; compounds contribute their stored `depth`. For any valid v0.2 compound, `depth == body_level` numerically.

A formula that diverged from `body_level` was considered (e.g. the v0.1 `order` formula that climbed only when at least two inputs were peers at the max). It was rejected because the resulting integer was path-dependent — composing the same atoms via different bracketings produced different `order` values, and no operator in the codebase consumed `order` semantically. `depth = body_level` is a clean, predictable metric whose value is intrinsic to the artifact's content tree.

---

## The delimiter scheme

Compound bodies are sequences of K complete sub-files, each wrapped by an open/close delimiter pair on its own line. The number of tildes scales with `body_level`.

### Formula

For an element with `body_level = N`, the delimiter pair is:

```
open  = (N+1) tildes + ">>"        e.g. body_level=1 → "~~>>" ; body_level=2 → "~~~>>"
close = (N+1) tildes + "<<"        e.g. body_level=1 → "~~<<" ; body_level=2 → "~~~<<"
```

### Table

| `body_level` | Open                 | Close                |
|--------------|----------------------|----------------------|
| 1            | `~~>>`               | `~~<<`               |
| 2            | `~~~>>`              | `~~~<<`              |
| 3            | `~~~~>>`             | `~~~~<<`             |
| N            | `~`×(N+1) + `>>`     | `~`×(N+1) + `<<`     |

### Properties

The scheme has four deliberate properties:

1. **Chiral**: the open and close delimiters are visually distinct (`>>` vs `<<`). A close cannot be misread as an open. The parser enforces matched pairs.

2. **Monotonic with `body_level`**: a level-`N` delimiter has *strictly more* tildes than any level less than `N`. This is the escalation rule.

3. **Doesn't collide with Markdown**: standard CommonMark uses three backticks ` ``` ` or three tildes `~~~` for code fences. The minimum `body_level = 1` uses 2 tildes plus `>>`/`<<` — neither pattern overlaps. Higher levels add more tildes; the `>>`/`<<` suffix further distinguishes them from any code-fence usage.

4. **Each chunk is itself a complete file**: between an open and its matched close is the full text of one complete Oovra `.md` file — frontmatter, blank line, body, all preserved. This is what makes `decompose --full` losslessly recover every original.

### Each chunk structure

A single chunk between an open/close delimiter pair looks like:

```
~~>>
+++
name = "..."
kind = "atom"
id = "..."
version = "..."
meta = "..."
+++

<the input's body>
~~<<
```

The leading and trailing newlines around `+++` belong to the embedded file; the delimiter lines are their own lines outside the file content.

### Parser behavior

When `decompose` is called on a compound with `body_level = N`:

1. Read `N` from frontmatter.
2. Compute the open and close strings: `~×(N+1) + ">>"` and `~×(N+1) + "<<"`.
3. Scan the body line by line:
   - On a line matching `open` exactly → start collecting a new chunk
   - On a line matching `close` exactly → finalize the current chunk
   - Any other line → if inside a chunk, append to it; otherwise ignore (lines between `<<` and the next `>>` are inter-chunk whitespace)
4. Each finalized chunk is fed to `parse()` as a complete Oovra file.

**Lines that *almost* match are inert**: if the body contains a literal `~~~>>` (3 tildes) line and `body_level = 1` (expecting 2-tilde delimiters), the 3-tilde line is just text from the parser's perspective. This is the property that makes nesting safe.

---

## Worked example: a depth-3 compound

To make the formulas concrete, here's how `body_level` builds up to 3 levels.

**Construction**:

- Atoms: `a`, `b`, `c`, `d`, `e`, `f` (`body_level` absent, contributing 0)
- Build two compounds-of-atoms:
  - `sub-1 = compose(a, b, c)` → `body_level = 1`, `depth = 1`
  - `sub-2 = compose(d, e, f)` → `body_level = 1`, `depth = 1`
- Build a compound-of-compounds:
  - `mid-1 = compose(sub-1, sub-2)` → `body_level = 2`, `depth = 2`
- Build another compound-of-compounds the same way:
  - `mid-2 = compose(sub-3, sub-4)` → `body_level = 2`, `depth = 2`
- Build the root:
  - `root = compose(mid-1, mid-2)` → `body_level = 3`, `depth = 3`

**On-disk structure of `root`**:

```
+++
... root's frontmatter ...
kind = "compound"
body_level = 3
depth = 3
composed_of = [{id="mid-1", version="1.0.0"}, {id="mid-2", version="1.0.0"}]
+++

~~~~>>                              <- level-3 delimiter (4 tildes + >>)
+++
... mid-1's frontmatter (kind = "compound", body_level = 2) ...
+++

~~~>>                               <- level-2 delimiter (3 tildes + >>) wrapping mid-1's children
+++
... sub-1's frontmatter (kind = "compound", body_level = 1) ...
+++

~~>>                                <- level-1 delimiter (2 tildes + >>) wrapping sub-1's children
+++
... a's frontmatter (kind = "atom") ...
+++

<a's body>
~~<<
~~>>
+++
... b's frontmatter (kind = "atom") ...
+++

<b's body>
~~<<
~~>>
+++
... c's frontmatter (kind = "atom") ...
+++

<c's body>
~~<<
~~~<<                               <- closing level-2 (for sub-1)
~~~>>                               <- opening level-2 (for sub-2)
... (similar nesting for sub-2) ...
~~~<<
~~~~<<                              <- closing level-3 (for mid-1)
~~~~>>                              <- opening level-3 (for mid-2)
... (similar nesting for mid-2) ...
~~~~<<
```

Three levels of delimiters, each strictly greater in tilde count than any inner level. The depth-3 parser scans for `~~~~>>` (4 tildes) — the `~~~>>` (3 tildes) and `~~>>` (2 tildes) lines are inert from its perspective.

This is why the parser is O(file size) and unambiguous regardless of compositional depth: each level peels off one wrapper, and the parse-then-recurse pattern handles all the nesting automatically.

---

## The mixed-kind regression

A historical note: in v0.1, the on-disk delimiter level was derived from a different formula than `body_level` is computed by today. Composing one compound with several atoms used to produce a body where the outer delimiters could collide with the inner compound's delimiters. The fix preserved in `compute_body_level` is to *always* compute `max(input.body_level) + 1`, regardless of how many inputs are at the max. See [demos/05-mixed-order-regression](../demos/05-mixed-order-regression/) for the worked demonstration and the in-tree regression test (`mixed_kind_compose_does_not_collide_with_inner_delimiters`).

---

## See also

- [schema.md](./schema.md) — where `body_level` and `depth` live in the frontmatter
- [command-compose.md](./command-compose.md) — how Compose computes them at composition time
- [command-decompose.md](./command-decompose.md) — how Decompose uses `body_level` to split bodies
- [demos/05-mixed-order-regression](../demos/05-mixed-order-regression/) — the historical collision case, now a regression test
- [v0.2 SPEC](../version-reports/v0.1/SPEC-v0.2.md) — the migration rationale for removing `order`
