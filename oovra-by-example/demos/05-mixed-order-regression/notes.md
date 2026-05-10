# Demo 5 — Mixed-Order Regression Case (the body_level escalation fix)

## What this demonstrates

A specific case that used to silently produce broken files: **composing a mix of order-1 and order-0 inputs where the order-1 has no peer**. The user's `compute_order` formula keeps the output's *logical* order at 1 in this case (because `count_at_max == 1` — only one input at the max=1 level, so no climb). But the *physical* body delimiter level must escalate anyway, or the outer parser collides with the inner element's delimiters.

This was a real bug caught during the live-test phase. It's now fixed, regression-tested, and demonstrated here.

## Steps that were run

1. Library has 4 atomics (re-used from demo 1).
2. Build `foundation.md` — an order-1 element from `role-statement` + `safety-fence` (homogeneous order-0 inputs, body_level=1, classic case).
3. Build `mixed-input-prompt.md` — order-1 element from `foundation` (order 1) + `tone-discipline` (order 0) + `format-rules` (order 0).
4. Inspect the outer file's `order` and `body_level` fields — they diverge.
5. Run `oovra decompose` to verify the parser handles the strict-escalation case.
6. Run `oovra decompose --full` and verify byte-equality of every leaf.

## The two integers diverge

```
mixed-input-prompt.md:
  order      = 1   (per user formula: max=1, count_at_max=1, no climb)
  body_level = 2   (per max(input.order) + 1 = 2)
```

This is the case the bug fix covers. Without the fix, `body_level` would have followed `order = 1`, and the outer body wrapper would have used 2-tilde delimiters — colliding with `foundation`'s body which also uses 2-tilde delimiters internally.

## Folder layout

```
05-mixed-order-regression/
├── library/
│   ├── role-statement.md           ← order 0
│   ├── safety-fence.md             ← order 0
│   ├── tone-discipline.md          ← order 0
│   ├── format-rules.md             ← order 0
│   ├── foundation.md               ← order 1, body_level 1 (homogeneous compose, classic case)
│   └── mixed-input-prompt.md       ← order 1, body_level 2 (mixed compose, the previously-broken case)
├── recovered/
│   └── mixed-input-prompt/
│       ├── mixed-input-prompt.md
│       ├── foundation/
│       │   ├── foundation.md
│       │   ├── role-statement.md   ← byte-identical to original
│       │   └── safety-fence.md     ← byte-identical to original
│       ├── tone-discipline.md       ← byte-identical to original
│       └── format-rules.md          ← byte-identical to original
├── delimiter-analysis.md            ← line counts proving strict escalation
└── decompose-outputs.md             ← raw outputs of decompose and decompose --full
```

## Two pre-fix vs post-fix outputs (architectural)

| | Pre-fix code | Post-fix code |
|---|---|---|
| Body delimiter level | `output.order` (= 1) | `compute_body_level(...)` (= 2) |
| Outer wrapper delimiter | `~~>>` / `~~<<` (2 tildes) | `~~~>>` / `~~~<<` (3 tildes) |
| Collision with `foundation`'s body | YES — `foundation`'s body has `~~>>` lines | NO — outer scans for 3 tildes, ignores 2-tilde lines |
| `oovra decompose` result | ERROR: `encountered '~~>>' while still inside an open chunk` | SUCCESS: 3 immediate inputs, byte-identical leaves |

## Why this case is structurally important

The user's `compute_order` formula expresses a **semantic** claim: "you only climb the order ladder when ≥2 peers exist at the max input level." That's a meaningful rule about when a composition is genuinely a level-up vs. just a wrapping. Preserving that semantic meant we couldn't simply make `output_order = max + 1` always.

But the **parser** needs strict-monotonic delimiters at every level — that's a physical requirement, not a semantic one. So we now store **two integers**: `order` (the user's logical depth) and `body_level` (the physical delimiter level, always `max + 1`). They coincide in the common homogeneous case (`compose([0, 0, 0])` → `order = 1, body_level = 1`) but diverge here. The schema, validator, and decompose logic all read `body_level` for delimiter decisions and `order` for everything else.

## Test coverage in the repo

`tests/end_to_end.rs::mixed_order_compose_does_not_collide_with_inner_delimiters` runs exactly this scenario as part of the test suite. Any future regression in body_level computation will be caught at `cargo test`.

## See also

- [reference/order-and-delimiters.md](../../reference/order-and-delimiters.md) — the two-integer system in full, with worked examples at every order
- [reference/command-compose.md § Output ordering](../../reference/command-compose.md#output-ordering-how-compute_order-and-body_level-are-determined) — how Compose computes both integers
- [reference/command-decompose.md § How decompose finds the delimiter level](../../reference/command-decompose.md#how-decompose-finds-the-delimiter-level) — why `body_level` (not `order`) drives splitting
- [reference/errors.md § BodyParse](../../reference/errors.md#bodyparse) — the error that fires when delimiters collide
