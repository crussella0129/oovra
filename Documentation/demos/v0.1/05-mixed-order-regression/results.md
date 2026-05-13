# Demo 5 — Results

## The two integers

| Field | Value | Determined by |
|---|---|---|
| `order` | 1 | User formula: `count_at_max > 1 ? max+1 : max` (here count=1, so order stays at max=1) |
| `body_level` | 2 | Physical: `max(input.order) + 1` (always escalates strictly) |

## Decompose verdict

| Operation | Result |
|---|---|
| `oovra decompose` (one-level) | 3 immediate inputs recovered: foundation (order 1), tone-discipline (order 0), format-rules (order 0) |
| `oovra decompose --full` | folder tree with 4 leaves; foundation got its own subdirectory because it is order >= 1 |

## Byte-equality of recovered leaves

| Leaf path in recovered tree | Versus original | Result |
|---|---|---|
| `foundation/role-statement.md` | `library/role-statement.md` | BYTE-IDENTICAL |
| `foundation/safety-fence.md` | `library/safety-fence.md` | BYTE-IDENTICAL |
| `tone-discipline.md` | `library/tone-discipline.md` | BYTE-IDENTICAL |
| `format-rules.md` | `library/format-rules.md` | BYTE-IDENTICAL |

## Delimiter line counts

| Level | Tildes | Open count | Close count | Used by |
|---|---|---|---|---|
| 2 (outer) | 3 | 3 | 3 | the outer body wrapper for 3 immediate inputs |
| 1 (inner) | 2 | 2 | 2 | inside foundation's embedded body |

## Conclusion

The previously-broken mixed-order case now produces a correctly-formed file with strictly-monotonic delimiter levels, and decompose recovers all leaves byte-identically. Verified against the in-tree regression test `mixed_order_compose_does_not_collide_with_inner_delimiters`.
