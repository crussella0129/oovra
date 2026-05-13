# Demo 5 — Delimiter Lines in `mixed-input-prompt.md`

This file is order=1, body_level=2. The body uses 3-tilde delimiters (`~~~>>` / `~~~<<`)
because one of its inputs (`foundation`) is itself an order-1 element with body_level=1,
whose body uses 2-tilde delimiters. To avoid collision, the outer wrapper must use *more*
tildes than any inner delimiter.

## Delimiter line counts

```
Outer level-2 (~~~>>): 3 opens (matches the 3 immediate inputs)
Outer level-2 (~~~<<): 3 closes
Inner level-1 (~~>>):  2 opens (only inside foundation's body, which embeds 2 atomics)
Inner level-1 (~~<<):  2 closes
```

## Why this used to break

Before the fix, the body delimiter level was determined by the output `order`. The user formula
returns `output_order = 1` here (count_at_max == 1, no climb). So the outer wrapper would have used
**level-1 (2-tilde) delimiters**. But `foundation` (one of the inputs) is itself an order-1 file
whose body contains **level-1 (2-tilde) delimiters**. The outer parser, scanning for level-1 delimiters
in mixed-input-prompt.md's body, would have falsely matched foundation's inner delimiters and split
the chunk in the wrong place. `decompose` would then fail with `encountered ~~>> while still inside
an open chunk`.

## Why it works now

The fix decoupled `body_level` from `order`. `body_level` is computed independently as
`max(input.order) + 1` regardless of the order formula. Here that gives `body_level = 2`
(3-tilde delimiters), strictly greater than any input body_level. The outer parser scans for
*exactly* 3 tildes, finds the 3 expected matches, and ignores the 2-tilde inner delimiters as
inert text. Decompose succeeds.
