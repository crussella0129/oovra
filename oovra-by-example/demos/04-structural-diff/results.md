# Demo 4 — Results

## Surface (byte-level) difference

Lines reported by raw `diff pairing-v1.md pairing-v2.md`: **26**

Most of these are noise: timestamp drift, version strings repeated in the recipe and the embedded headers, prose changes, etc.

## Structural diff result

`oovra compare` reduces those 26 lines of surface noise to:

| Category | Count | Details |
|---|---|---|
| Inputs added | 0 | (recipes have identical input sets) |
| Inputs removed | 0 | |
| Inputs version-changed | 2 | `role-statement: 1.0.0 → 2.0.0`, `format-rules: 1.0.0 → 1.1.0` |

## What changed in library-v2

| File | Change |
|---|---|
| `role-statement.md` | "1.0.0" → "2.0.0" |
| `safety-fence.md` | unchanged |
| `tone-discipline.md` | unchanged |
| `format-rules.md` | "1.0.0" → "1.1.0" |

## Conclusion

Two prompts whose on-disk bytes differ across 26 lines were correctly identified as "same recipe, two version bumps." This is the diff that surfaces the actual semantic change for code review, dependency tracking, and dedup analysis.
