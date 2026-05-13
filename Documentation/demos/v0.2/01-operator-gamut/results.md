# Results — Full Operator Gamut

Captured outputs from running the four operators end-to-end against the fresh
fixtures in this folder. Every block is the literal CLI output (modulo ANSI
color stripped from human-format snippets). Hashes are SHA256 of the on-disk
file bytes.

Generated against **Oovra v0.2.0** binary at `target/debug/oovra`, building
from commit `3b34b25` (the v0.2 release commit).

---

## 1. Create — five atoms scaffolded

CLI output:

```
Created Documentation/demos/v0.2/01-operator-gamut/library-v1/role-statement.md
Created Documentation/demos/v0.2/01-operator-gamut/library-v1/refusal-policy.md
Created Documentation/demos/v0.2/01-operator-gamut/library-v1/tone-direct.md
Created Documentation/demos/v0.2/01-operator-gamut/library-v1/examples-block.md
Created Documentation/demos/v0.2/01-operator-gamut/library-v1/format-rules.md
```

On-disk atom files (after the post-scaffold body edits that replaced the
scaffold TODO comment with real prompt content):

| File                          | Size      | SHA256                                                               |
|-------------------------------|-----------|----------------------------------------------------------------------|
| `library-v1/role-statement.md`  | 348 bytes | `9b305f13b3cad0c184b425072e4574d0dc9984fbdb6d0ef0c03dd69a8e92a8ac`   |
| `library-v1/refusal-policy.md`  | 426 bytes | `5d6edaaeed12062ac4c2ac69572460935718419f3e2f34bda2425f7708d3e106`   |
| `library-v1/tone-direct.md`     | 398 bytes | `90e4932c9c6bfdfe9a0f30d9df9ebf934ced08a41336076ff97866f87f9f93c9`   |
| `library-v1/examples-block.md`  | 434 bytes | `484c5b8085485860edef878ce82c80def1a28ddc6ecf8e43cd4887bfa0faae07`   |
| `library-v1/format-rules.md`    | 471 bytes | `9c74ba63a6bfa471aaeba7bc223e3e53b9ee1c4470e59ef77b9f30ae90a71a9f`   |
| `library-v2/refusal-policy.md`  | 584 bytes | `6ee4804cacecad36cd21d70188ea832aa0654570942899841337608dc7512877`   |

`library-v2/refusal-policy.md` is the version-bumped 2.0.0 atom. The other
four files in `library-v2/` are byte-identical to their `library-v1/`
counterparts (copies, unmodified).

---

## 2. Compose — two compounds produced

CLI output:

```
Composed Documentation/demos/v0.2/01-operator-gamut/outputs/coding-agent-v1.md (compound, body_level 1, 4 inputs)
Composed Documentation/demos/v0.2/01-operator-gamut/outputs/coding-agent-v2.md (compound, body_level 1, 4 inputs)
```

| File                              | Size       | SHA256                                                               |
|-----------------------------------|------------|----------------------------------------------------------------------|
| `outputs/coding-agent-v1.md`      | 2122 bytes | `c8ec0b9af2a1b82e0dafa145d5d38c61b131d62c2be6e3db31e68b99ec38c525`   |
| `outputs/coding-agent-v2.md`      | 2314 bytes | `5c707570e78ae28756f39098afdd11bd8fe7dc2395aed9b18e8dd16b21b06b56`   |

The two compounds share the same `id` (`coding-agent`) and the same `version`
(`1.0.0`) — they differ only in their `composed_of` recipes and the embedded
bodies. v2's larger size (192 bytes more) reflects the extra calibration
sentence in `refusal-policy@2.0.0` plus the format-rules atom that replaced
examples-block.

v1 frontmatter (head):

```toml
+++
name = "Coding Agent v1"
kind = "compound"
id = "coding-agent"
version = "1.0.0"
meta = "First composition: role+refusal+tone+examples"
generated_at = "2026-05-13T22:05:04.807372200+00:00"
render_mode = "markdown-h2"
body_level = 1
depth = 1

[[composed_of]]
id = "role-statement"
version = "1.0.0"
... (three more entries) ...
+++
```

v2 frontmatter (head):

```toml
+++
name = "Coding Agent v2"
kind = "compound"
id = "coding-agent"
version = "1.0.0"
meta = "Second composition: refusal+tone+role+format"
generated_at = "2026-05-13T22:05:04.847576900+00:00"
render_mode = "markdown-h2"
body_level = 1
depth = 1

[[composed_of]]
id = "refusal-policy"
version = "2.0.0"
... (three more entries) ...
+++
```

Note `body_level = 1` and `depth = 1` on both — they were built from
homogeneous atom inputs, so the body-delimiter level is the minimum (`~~>>`
/ `~~<<`, two-tilde chiral delimiters).

---

## 3. Decompose — lossless roundtrip on v1

### 3a. One-level human

```
Decompose coding-agent (1.0.0) — compound at body_level 1, 4 immediate input(s)
  - role-statement @ 1.0.0 (atom) — Role Statement
  - refusal-policy @ 1.0.0 (atom) — Refusal Policy
  - tone-direct    @ 1.0.0 (atom) — Tone: Direct
  - examples-block @ 1.0.0 (atom) — Examples Block
```

### 3b. One-level JSON

See [`outputs/decompose-v1.json`](./outputs/decompose-v1.json). Excerpt:

```json
{
  "element_id": "coding-agent",
  "element_kind": "compound",
  "element_version": "1.0.0",
  "body_level": 1,
  "render_mode": "markdown-h2",
  "inputs": [
    { "id": "role-statement", "version": "1.0.0", "kind": "atom", "name": "Role Statement" },
    { "id": "refusal-policy", "version": "1.0.0", "kind": "atom", "name": "Refusal Policy" },
    { "id": "tone-direct",    "version": "1.0.0", "kind": "atom", "name": "Tone: Direct" },
    { "id": "examples-block", "version": "1.0.0", "kind": "atom", "name": "Examples Block" }
  ]
}
```

### 3c. `--full` folder tree

CLI output:

```
Decomposed (full) Documentation/demos/v0.2/01-operator-gamut/outputs/coding-agent-v1.md -> Documentation/demos/v0.2/01-operator-gamut/outputs/decompose-tree/coding-agent
```

Tree structure:

```
outputs/decompose-tree/coding-agent/
├── coding-agent.md             ← the compound itself
├── role-statement.md           ← recovered atom
├── refusal-policy.md           ← recovered atom
├── tone-direct.md              ← recovered atom
└── examples-block.md           ← recovered atom
```

### 3d. Roundtrip byte-identity

```
$ diff -q outputs/decompose-tree/coding-agent/role-statement.md library-v1/role-statement.md
$ diff -q outputs/decompose-tree/coding-agent/refusal-policy.md library-v1/refusal-policy.md
$ diff -q outputs/decompose-tree/coding-agent/tone-direct.md    library-v1/tone-direct.md
$ diff -q outputs/decompose-tree/coding-agent/examples-block.md library-v1/examples-block.md
(all four diff invocations: empty output = byte-identical)
```

Hash comparison:

| Atom              | Original SHA256          | Recovered SHA256         | Match |
|-------------------|--------------------------|--------------------------|-------|
| role-statement    | `9b305f13b3cad0...`      | `9b305f13b3cad0...`      | ✅ |
| refusal-policy    | `5d6edaaeed1206...`      | `5d6edaaeed1206...`      | ✅ |
| tone-direct       | `90e4932c9c6bfd...`      | `90e4932c9c6bfd...`      | ✅ |
| examples-block    | `484c5b8085485860...`    | `484c5b8085485860...`    | ✅ |

All four atoms recovered byte-for-byte from the compound's body alone — no
library access needed. The self-extracting-archive property holds for v0.2
compounds the same way it held for v0.1.

---

## 4. Compare — v1 vs v2 — all four diff axes fire

### 4a. Human format

```
Compare coding-agent <-> coding-agent  (compounds, structural diff)
  added inputs:
    + [3] format-rules @ 1.0.0
  removed inputs:
    - [3] examples-block @ 1.0.0
  version-changed inputs:
    ~ refusal-policy : 1.0.0 -> 2.0.0
  moved inputs:
    ↔ refusal-policy @ 2.0.0 : pos 1 -> pos 0
    ↔ tone-direct    @ 1.0.0 : pos 2 -> pos 1
    ↔ role-statement @ 1.0.0 : pos 0 -> pos 2
```

### 4b. JSON format

See [`outputs/compare.json`](./outputs/compare.json) for the full content.

```json
{
  "kind": "structural",
  "a_id": "coding-agent",
  "b_id": "coding-agent",
  "added": [
    { "position": 3, "input": { "id": "format-rules", "version": "1.0.0" } }
  ],
  "removed": [
    { "position": 3, "input": { "id": "examples-block", "version": "1.0.0" } }
  ],
  "version_changed": [
    { "id": "refusal-policy", "before_version": "1.0.0", "after_version": "2.0.0" }
  ],
  "moved": [
    { "id": "refusal-policy", "version": "2.0.0", "before_pos": 1, "after_pos": 0 },
    { "id": "tone-direct",    "version": "1.0.0", "before_pos": 2, "after_pos": 1 },
    { "id": "role-statement", "version": "1.0.0", "before_pos": 0, "after_pos": 2 }
  ],
  "recipes_equal": false
}
```

### 4c. Verdict against the spec

| Spec promise (SPEC §6, `command-compare.md` §"Outcome 2")     | Verified? |
|---------------------------------------------------------------|-----------|
| `added` reports new-side inputs with positions                | ✅ `format-rules` at pos 3 |
| `removed` reports old-side inputs with positions              | ✅ `examples-block` at pos 3 |
| `version_changed` reports same-id different-version           | ✅ `refusal-policy` 1.0.0 → 2.0.0 |
| `moved` reports same-id same-version different-position       | ✅ 3 entries |
| `version_changed` and `moved` are not mutually exclusive      | ✅ `refusal-policy` fires on **both** |
| `recipes_equal` only true when all four lists empty           | ✅ `false` here |

The `recipes_equal: false` is the load-bearing result. In v0.1 with the
order-blind diff, the same pair of compounds would have reported
`recipes_equal: true` because the set-keyed-by-id comparison ignored position
changes. The v0.2 sequence-aware compare correctly surfaces the reorder.

---

## 5. Reproducibility

To reproduce this demo end-to-end (replacing `$DEMO` with this folder):

```sh
# Step 1: scaffold the five atoms (then hand-edit each body)
for id in role-statement refusal-policy tone-direct examples-block format-rules; do
  oovra create --new $id --library $DEMO/library-v1
done

# Step 2: stage library-v2 (copy v1, bump refusal-policy)
cp -r $DEMO/library-v1/* $DEMO/library-v2/
# (then hand-edit $DEMO/library-v2/refusal-policy.md to version = "2.0.0")

# Step 3: compose v1 and v2
oovra compose --library $DEMO/library-v1 --out-id coding-agent \
  -o $DEMO/outputs/coding-agent-v1.md \
  role-statement refusal-policy tone-direct examples-block
oovra compose --library $DEMO/library-v2 --out-id coding-agent \
  -o $DEMO/outputs/coding-agent-v2.md \
  refusal-policy tone-direct role-statement format-rules

# Step 4: decompose v1 (all three modes)
oovra decompose $DEMO/outputs/coding-agent-v1.md
oovra decompose $DEMO/outputs/coding-agent-v1.md --format json
oovra decompose --full $DEMO/outputs/coding-agent-v1.md -o $DEMO/outputs/decompose-tree

# Step 5: compare
oovra compare $DEMO/outputs/coding-agent-v1.md $DEMO/outputs/coding-agent-v2.md
oovra compare $DEMO/outputs/coding-agent-v1.md $DEMO/outputs/coding-agent-v2.md --format json
```

If `oovra migrate` or `--legacy` is on a future test list, those are
exercised in `tests/end_to_end.rs::migrate_*` rather than as a demo here.

---

## See also

- [notes.md](./notes.md) — narrative explanation of what each step does and why
- [outputs/](./outputs/) — the captured artifacts referenced above
- [../../../reference/command-compare.md](../../../reference/command-compare.md) — full structural-diff reference
- [../../../../CHANGELOG.md](../../../../CHANGELOG.md) — v0.2 change list
