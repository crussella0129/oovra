# Demo 3 — Body Structure of `senior-pair-deep.md` (order 2)

The body of an order-2 file contains 2 order-1 sub-elements wrapped in level-2 delimiters.
Each order-1 sub-element has its own body containing 3 order-0 leaves wrapped in level-1 delimiters.
The strict tilde-count escalation (3 tildes outer vs 2 tildes inner) prevents collisions.

## Delimiter line counts in the order-2 file

```
Level-2 (~~~>>): 2 opens
Level-2 (~~~<<): 2 closes
Level-1 (~~>>):  6 opens (these are inside the order-1 sub-elements)
Level-1 (~~<<):  6 closes
```

## Embedded element IDs visible in the body

```
id = "senior-pair-deep"
id = "agent-foundations"
id = "output-discipline"
id = "agent-foundations"
id = "role-statement"
id = "safety-fence"
id = "tone-discipline"
id = "role-statement"
id = "safety-fence"
id = "tone-discipline"
id = "output-discipline"
id = "format-rules"
id = "examples-discipline"
id = "file-citation-rules"
id = "format-rules"
id = "examples-discipline"
id = "file-citation-rules"
```
