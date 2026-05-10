# Order and Delimiters — Specification

Oovra has **two distinct integers** that look similar but answer different questions:

- **`order`** — the *logical compositional depth*, user-facing, derived from a count-based formula.
- **`body_level`** — the *physical body-delimiter level*, parser-facing, always strictly escalating.

Most of the time they coincide. They diverge in mixed-order compositions, and that divergence is what keeps the on-disk format unambiguous. This page is the full specification of both, plus the chiral delimiter scheme they govern.

---

## The two integers, side by side

| Aspect            | `order`                                                                          | `body_level`                                                                       |
|-------------------|----------------------------------------------------------------------------------|------------------------------------------------------------------------------------|
| Stored on         | every file's frontmatter (always required)                                       | only composed files' frontmatter (required when `composed_of` is present)          |
| Source of value   | computed from input orders by [`compute_order`](#computing-order)                | computed from input orders by [`compute_body_level`](#computing-body_level)        |
| Range             | non-negative integer                                                             | non-negative integer ≥ 1 for composed; absent for atomic                            |
| What it means     | "how many distinct levels of peer composition went into producing this artifact" | "how many tildes the body-delimiter parser should look for"                         |
| Audience          | humans, semantic queries, the user                                               | the parser                                                                          |
| When they coincide | inputs are all at the same order (the common case)                              | same                                                                                |
| When they diverge | when only ONE input is at the max-order level (`count_at_max == 1`)              | `body_level` always escalates above `max(input.order)` even when `order` doesn't    |

---

## Computing `order`

The user-facing logical compositional depth. The rule:

```
let H = max(input.order for input in inputs)
let C = count of inputs whose order == H
output.order = if C > 1 then H + 1 else H
```

In words: you climb the order ladder **only when at least two inputs are peers at the maximum input order**. A single high-order input plus a bunch of lower-order inputs keeps you at the high-order's level — no climb, because there's no genuine peer-composition happening at the top.

### Worked examples

| Inputs                       | Max H | Count at H | Result | Reasoning                                                                |
|------------------------------|-------|------------|--------|--------------------------------------------------------------------------|
| 3 × order 0                  | 0     | 3          | 1      | Three peers at level 0, promote to 1                                       |
| 2 × order 1                  | 1     | 2          | 2      | Two peers at level 1, promote to 2                                         |
| 1 × order 0                  | 0     | 1          | 0      | Single input, no climb (degenerate; rarely useful)                         |
| 1 × order 1 + 4 × order 0    | 1     | 1          | 1      | Only one input at the max (1); no climb                                    |
| 1 × order 2 + 1 × order 1 + 5 × order 0 | 2 | 1     | 2      | Only one input at the max (2); no climb                                    |
| 2 × order 2 + 1 × order 1    | 2     | 2          | 3      | Two peers at level 2, promote to 3                                         |
| 3 × order 1 + 2 × order 0    | 1     | 3          | 2      | Three peers at level 1, promote to 2 (lower-order inputs don't change result) |

### Why `count > 1` and not `>= 1`

With `>= 1`, every Compose call would promote the order. Feeding one order-1 plus a hundred order-0's would yield order-2 — which is meaningless because no actual peer composition happened at the top level. The `> 1` rule correctly says "you only climb when at least two genuine peers exist at the max."

This makes `order` a real measure of compositional depth, not a counter of flat aggregations.

---

## Computing `body_level`

The physical on-disk delimiter level. The rule, much simpler:

```
output.body_level = max(input.order for input in inputs) + 1
```

That's it. The output's body uses a delimiter level **strictly greater than any input's `order`**. There's no count-based check, no conditional, no formula branching — `body_level` always escalates.

### Worked examples

| Inputs                       | Max input.order | `body_level` |
|------------------------------|------------------|--------------|
| 3 × order 0                  | 0                | 1            |
| 2 × order 1                  | 1                | 2            |
| 1 × order 0                  | 0                | 1            |
| 1 × order 1 + 4 × order 0    | 1                | 2            |
| 1 × order 2 + 1 × order 1 + 5 × order 0 | 2     | 3            |
| 2 × order 2 + 1 × order 1    | 2                | 3            |
| 3 × order 1 + 2 × order 0    | 1                | 2            |

### Why `body_level` must always escalate

The body of a composed file embeds each input as a **complete sub-file** (frontmatter + body), wrapped in delimiters. If an input is itself composed, its body contains delimiters from *its own* `body_level`. For the outer parser to unambiguously find its delimiters and not accidentally match the inner ones, the outer delimiter must be **strictly different** (specifically, *more tildes*) than any inner one.

Since the deepest inner delimiter has level `max(input.order)` (every composed file uses its own order as its `body_level` if it itself was a homogeneous composition), the outer level must be `>= max(input.order) + 1`. Adding 1 is the minimum-bump that guarantees no collision.

---

## When `order` and `body_level` diverge

```
output.order      = (count_at_max > 1) ? max + 1 : max
output.body_level = max + 1
```

They diverge **exactly when `count_at_max == 1`**, in which case `order == max` but `body_level == max + 1`.

### Why both exist as separate fields

A single integer can't serve both purposes:

- If we used the count-based formula for both, mixed-order compositions where `count_at_max == 1` would produce a `body_level` that collides with an inner input's delimiters. → parsing bug
- If we used `max + 1` for both, the user's semantic "you only climb with ≥2 peers" rule would be lost. → meaningful concept gone

Storing both as separate fields keeps both correct. The downside is a small amount of redundancy in the frontmatter and a slightly more complex schema. The upside is that the parser is unambiguous and the user-facing semantics are preserved.

### Worked example of the divergence (this was a real bug pre-fix)

`compose([order-1-element, atomic-A, atomic-B])`:

- Max input order: `1`
- Count at max: `1`
- → `order = 1`, `body_level = 2`

The output's body uses **3-tilde delimiters** (`~~~>>`, `~~~<<`). Inside that body, the order-1 input has its own embedded body using **2-tilde delimiters** (`~~>>`, `~~<<`). Because 3 > 2, the outer parser scanning for 3-tilde delimiters never matches the 2-tilde inner ones.

Before the fix that decoupled the two integers, the outer file used `output.order = 1` for its delimiter level — same as the inner — causing a collision. `decompose` would emit `encountered '~~>>' while still inside an open chunk`. See [demos/05-mixed-order-regression](../demos/05-mixed-order-regression/) for the full demonstration and the in-tree regression test.

---

## The delimiter scheme

Composed-element bodies are sequences of K complete sub-files, each wrapped by an open/close delimiter pair on its own line. The number of tildes scales with `body_level`.

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
order = 0
id = "..."
version = "..."
meta = "..."
+++

<the input's body>
~~<<
```

The leading and trailing newlines around `+++` belong to the embedded file; the delimiter lines are their own lines outside the file content.

### Parser behavior

When `decompose` is called on a file with `body_level = N`:

1. Read `N` from frontmatter.
2. Compute the open and close strings: `~×(N+1) + ">>"` and `~×(N+1) + "<<"`.
3. Scan the body line by line:
   - On a line matching `open` exactly → start collecting a new chunk
   - On a line matching `close` exactly → finalize the current chunk
   - Any other line → if inside a chunk, append to it; otherwise ignore (lines between `<<` and the next `>>` are inter-chunk whitespace)
4. Each finalized chunk is fed to `parse()` as a complete Oovra file.

**Lines that *almost* match are inert**: if the body contains a literal `~~~>>` (3 tildes) line and `body_level = 1` (expecting 2-tilde delimiters), the 3-tilde line is just text from the parser's perspective. This is the property that makes nesting safe.

---

## Worked example: an order-3 composition

To make all of this concrete, here's how the formulas apply to an order-3 element.

**Construction**:

- Atomics: `a`, `b`, `c`, `d`, `e`, `f`, `g`, `h` (all order 0)
- Build two order-1 sub-prompts:
  - `sub-1 = compose(a, b, c)` → `order=1`, `body_level=1`
  - `sub-2 = compose(d, e, f)` → `order=1`, `body_level=1`
- Build an order-2 sub-prompt:
  - `mid-1 = compose(sub-1, sub-2)` → `order=2`, `body_level=2`
- Build another order-2 sub-prompt:
  - `mid-2 = compose(g, h)` plus some other peer... wait, we need a second order-2. Let me redo: assume `mid-2 = compose(sub-3, sub-4)` for some other order-1s.
- Build the order-3 root:
  - `root = compose(mid-1, mid-2)` → max=2, count=2, **`order=3`**, **`body_level=3`**

**On-disk structure of `root`**:

```
+++
... root's frontmatter ...
order = 3
body_level = 3
composed_of = [{id="mid-1", version="1.0.0"}, {id="mid-2", version="1.0.0"}]
+++

~~~~>>                              <- level-3 delimiter (4 tildes + >>)
+++
... mid-1's frontmatter (order=2, body_level=2) ...
+++

~~~>>                               <- level-2 delimiter (3 tildes + >>) wrapping mid-1's children
+++
... sub-1's frontmatter (order=1, body_level=1) ...
+++

~~>>                                <- level-1 delimiter (2 tildes + >>) wrapping sub-1's children
+++
... a's frontmatter (order=0) ...
+++

<a's body>
~~<<
~~>>
+++
... b's frontmatter (order=0) ...
+++

<b's body>
~~<<
~~>>
+++
... c's frontmatter (order=0) ...
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

Three levels of delimiters, each strictly greater in tilde count than any inner level. The order-3 parser scans for `~~~~>>` (4 tildes) — the `~~~>>` (3 tildes) and `~~>>` (2 tildes) lines are inert from its perspective.

This is why the parser is O(file size) and unambiguous regardless of compositional depth: each level peels off one wrapper, and the parse-then-recurse pattern handles all the nesting automatically.

---

## See also

- [schema.md](./schema.md) — where `order` and `body_level` live in the frontmatter
- [command-compose.md](./command-compose.md) — how Compose computes both at composition time
- [command-decompose.md](./command-decompose.md) — how Decompose uses `body_level` to split bodies
- [demos/05-mixed-order-regression](../demos/05-mixed-order-regression/) — the divergent case as a worked demo
