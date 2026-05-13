# Demo: Full Operator Gamut (v0.2)

## What this demonstrates

End-to-end exercise of all four Oovra operators (**Create**, **Compose**, **Decompose**, **Compare**) using the v0.2 CLI against fresh fixtures. The demo doubles as the integration test for v0.2's headline new feature — **sequence-aware compare** — by producing two compounds that differ on all four diff axes (`added`, `removed`, `version_changed`, `moved`) simultaneously.

This was not synthesized. Every artifact in this folder was produced by running the actual `oovra` binary against the actual filesystem; the captured outputs in [`outputs/`](./outputs/) are byte-for-byte what the CLI emitted.

## Steps that were run

### 1. Create (scaffold five atoms)

```sh
oovra create --new role-statement   --library .../library-v1 --name "Role Statement"
oovra create --new refusal-policy   --library .../library-v1 --name "Refusal Policy"
oovra create --new tone-direct      --library .../library-v1 --name "Tone: Direct"
oovra create --new examples-block   --library .../library-v1 --name "Examples Block"
oovra create --new format-rules     --library .../library-v1 --name "Format: Markdown"
```

Each command writes a v0.2 atom file (frontmatter contains `kind = "atom"`, no `order`). Bodies were edited from the scaffolded TODO comment to real prompt prose. Five atoms total, all at `version = "1.0.0"`.

### 2. Stage library-v2 with a version-bumped atom

`library-v2/` was created by copying `library-v1/` and bumping `refusal-policy` to `version = "2.0.0"` with an extra calibration sentence. All other atoms remain at `1.0.0`. This lets the compare step exercise a `version_changed` diff axis without changing identities.

### 3. Compose (build two compounds)

```sh
oovra compose --library .../library-v1 -o .../outputs/coding-agent-v1.md \
  --out-id coding-agent --out-name "Coding Agent v1" \
  role-statement refusal-policy tone-direct examples-block

oovra compose --library .../library-v2 -o .../outputs/coding-agent-v2.md \
  --out-id coding-agent --out-name "Coding Agent v2" \
  refusal-policy tone-direct role-statement format-rules
```

| Compound | Inputs (in order)                                                | Library |
|----------|------------------------------------------------------------------|---------|
| v1       | `role-statement, refusal-policy, tone-direct, examples-block`    | library-v1 (all 1.0.0) |
| v2       | `refusal-policy, tone-direct, role-statement, format-rules`      | library-v2 (refusal-policy bumped to 2.0.0) |

The difference profile across the two compounds:

- **added**: `format-rules` (in v2, not in v1)
- **removed**: `examples-block` (in v1, not in v2)
- **version_changed**: `refusal-policy` (1.0.0 → 2.0.0)
- **moved**: `role-statement`, `refusal-policy`, `tone-direct` (each at a different position)

This is the worst-case input for compare — every axis fires, and `refusal-policy` fires on both `version_changed` and `moved` because it was both bumped and reordered. The spec-promised non-exclusivity of those two axes is exercised.

### 4. Decompose (lossless roundtrip on v1)

Three modes were run:

```sh
oovra decompose .../outputs/coding-agent-v1.md                  # one-level human report
oovra decompose .../outputs/coding-agent-v1.md --format json    # one-level JSON
oovra decompose --full .../outputs/coding-agent-v1.md -o .../outputs/decompose-tree
```

The `--full` invocation produced a folder tree at `outputs/decompose-tree/coding-agent/` containing the compound's file plus one file per atom leaf. Each recovered atom was then `diff`'d against the original in `library-v1/`:

```sh
diff -q outputs/decompose-tree/coding-agent/role-statement.md library-v1/role-statement.md
# (empty output = byte-identical)
```

All four recovered atoms were byte-identical to their library-v1 originals. The compound carries enough information in its body alone to reconstruct every leaf without consulting the library — the same self-extracting-archive property that demo 01 verified for v0.1.

### 5. Compare (the four-axis structural diff)

```sh
oovra compare .../outputs/coding-agent-v1.md .../outputs/coding-agent-v2.md
oovra compare .../outputs/coding-agent-v1.md .../outputs/coding-agent-v2.md --format json
```

See [`results.md`](./results.md) for the captured outputs. The diff correctly surfaces all four axes:

- **added**: `format-rules @ 1.0.0` at position 3
- **removed**: `examples-block @ 1.0.0` at position 3
- **version_changed**: `refusal-policy` 1.0.0 → 2.0.0
- **moved**: three entries (the three inputs present in both compounds with shuffled positions)

`recipes_equal: false`.

## What this exercises that the unit tests don't

Unit tests cover each operator in isolation against synthetic in-memory fixtures. This demo:

- **Exercises the CLI surface**, not just the library API — every step is a real `oovra` invocation through `clap`'s argument parsing.
- **Round-trips files through disk** at each step — the v1 compound is decomposed from its on-disk form, not from an in-memory `PromptElement`. This catches any drift in `element::write` → `element::parse_file` that an in-memory test would miss.
- **Combines all four diff axes in one comparison.** The unit tests assert each axis individually (`compare_detects_reorder`, `compare_distinguishes_move_from_add_remove`, `compare_reports_pure_version_change_not_as_move`); this demo confirms they compose correctly when fired together.
- **Demonstrates that `version_changed` and `moved` are not mutually exclusive.** `refusal-policy` appears on both lists — bumped *and* reordered — which is the spec-promised behavior at §3.4 of `command-compare.md` and SPEC §6.

## Folder layout

```
01-operator-gamut/
├── notes.md                          ← this file
├── results.md                        ← captured outputs from every operator
├── library-v1/                       ← five atoms at version 1.0.0
│   ├── role-statement.md
│   ├── refusal-policy.md
│   ├── tone-direct.md
│   ├── examples-block.md
│   └── format-rules.md
├── library-v2/                       ← copy of library-v1 with refusal-policy bumped to 2.0.0
│   └── (same files; refusal-policy.md differs)
└── outputs/
    ├── coding-agent-v1.md            ← compose against library-v1 (4 inputs)
    ├── coding-agent-v2.md            ← compose against library-v2 (4 inputs, different mix)
    ├── compose-v1-text.md            ← `compose --text` flattened prose of v1
    ├── decompose-v1.json             ← `decompose --format json` of v1
    ├── decompose-tree/coding-agent/  ← `decompose --full` output tree
    └── compare.json                  ← `compare --format json` of v1 vs v2
```

## See also

- [results.md](./results.md) — the captured CLI outputs with timestamps and SHA256 sums
- [../../v0.1/04-structural-diff/](../../v0.1/04-structural-diff/) — the v0.1 baseline for structural diff (pre-sequence-aware)
- [../../../reference/command-compare.md](../../../reference/command-compare.md) — full reference for the kind-aware, sequence-aware compare
- [../../../../CHANGELOG.md](../../../../CHANGELOG.md) — what changed in v0.2
