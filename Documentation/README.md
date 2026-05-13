# Documentation

The complete user-facing and builder-facing documentation snapshot for Oovra v0.2. Three sibling sections, each serving a different audience:

| Section                                       | For                                                  | Style                                                                                                |
|-----------------------------------------------|------------------------------------------------------|------------------------------------------------------------------------------------------------------|
| [`reference/`](./reference/)                  | Users looking up "how do I do X?"                    | Feature-by-feature specifications — every command, every flag, every error variant, every schema field |
| [`demos/`](./demos/)                          | Users wanting to see "does this actually work?"      | End-to-end walkthroughs with concrete inputs, captured outputs, byte-equality verdicts                |
| [`reference/build-guide.md`](./reference/build-guide.md) | Builders/forkers wanting to "reimplement this"       | From-first-principles construction guide; the full Rust walkthrough                                   |

The v0.1-era planning documents (efficacy rating, ceiling analysis, v0.2 scoping, server feasibility, the v0.2 SPEC itself) live in [`version-reports/v0.1/`](./version-reports/v0.1/) as the historical record.

---

## I want to learn Oovra — where do I start?

The shortest path to understanding what the tool does:

1. **[`demos/01-lossless-roundtrip/notes.md`](./demos/01-lossless-roundtrip/notes.md)** and its [`results.md`](./demos/01-lossless-roundtrip/results.md) — concrete proof of the system's central claim that a compound is a self-extracting archive. ~5 minutes.
2. **[`demos/02-text-prose-output/notes.md`](./demos/02-text-prose-output/notes.md)** — see the two external presentations of a single Oovra file. ~5 minutes.
3. **[`reference/schema.md`](./reference/schema.md)** — the file format in detail. ~10 minutes.
4. **[`reference/command-compose.md`](./reference/command-compose.md)** — the central operation. ~10 minutes.
5. **Pick the rest as needed** — demos 3–5 for deeper architectural claims; remaining reference docs for command-by-command lookups.

If you've used the tool already and need to look something up: skip directly to the relevant doc in [`reference/`](./reference/) — its [`README.md`](./reference/README.md) is the per-doc index.

---

## I want to use Oovra — what's the workflow?

A typical session, with links to the relevant docs:

```bash
# 1. Author atoms (one per element)
oovra create --new role-statement --library ./prompts --name "Role Statement"
# ... edit ./prompts/role-statement.md to add your prompt text

# 2. Compose them into a compound
oovra compose --library ./prompts --out-id coding-agent --out-name "Coding Agent" \
  -o ./prompts/coding-agent.md \
  role-statement safety-fence tone-direct

# 3. Get a paste-ready prompt for your model
oovra compose --library ./prompts --text coding-agent
# → outputs clean Markdown with ## headers per element

# 4. After editing some atoms, see what changed structurally
oovra compare ./prompts-old/coding-agent.md ./prompts/coding-agent.md
# → shows added/removed/version-changed/moved inputs (v0.2 surfaces position changes)

# 5. Recover the inputs of any compound (even one you didn't compose yourself)
oovra decompose --full ./prompts/coding-agent.md -o ./out
# → writes the full input tree to ./out/coding-agent/
```

Each step is documented in detail in the corresponding `reference/command-*.md`.

### Migrating a v0.1 library

If your library was authored against Oovra v0.1 (with `order = N` frontmatter), migrate it in place:

```bash
oovra migrate ./prompts
# → walks recursively, rewrites every file to v0.2 schema, preserves generated_at
```

Run it in a clean Git working tree so the diff is auditable. See [`../CHANGELOG.md`](../CHANGELOG.md) for the full v0.1 → v0.2 change list.

---

## I want to extend or fork Oovra — what should I read?

For builders, [`reference/build-guide.md`](./reference/build-guide.md) walks through the full Rust implementation from scratch: schema design decisions, parser internals, the serde derive patterns, the validator's kind-aware dispatch, the body-delimiter scheme, the structural diff, the test strategy. It's written so that someone with basic Rust experience can reproduce the tool by following it.

The build guide is paired with [`reference/schema.md`](./reference/schema.md) (the contract the build guide implements). For the migration-specific rationale, see [`version-reports/v0.1/SPEC-v0.2.md`](./version-reports/v0.1/SPEC-v0.2.md).

---

## Snapshot status

This documentation snapshot was generated against **Oovra v0.2.0**. The 5 demos were originally captured against v0.1.0 and migrated in place by `oovra migrate`; their library files now carry `kind` discriminators, but the rendered outputs and recorded `results.md` content are unchanged because migration is a representation change, not a regeneration.

When the source code changes (new commands, new schema fields, new render modes, etc.), the corresponding docs in `reference/` are updated and the demos in `demos/` are re-run. Treat any file in here as a snapshot of the version it was generated against — see git history for the dated lineage.

---

## Structure at a glance

```
Documentation/
├── README.md                              ← you are here (the docs navigation hub)
│
├── reference/                              ← feature-by-feature specifications
│   ├── README.md                           ← reference index + reading order
│   ├── schema.md                           ← TOML schema: kind discriminator, every field, every invariant
│   ├── kind-and-delimiters.md              ← body_level, depth, and the chiral delimiter scheme
│   ├── command-create.md                   ← oovra create (--new and --label modes)
│   ├── command-compose.md                  ← oovra compose (file, --text, --re-render modes)
│   ├── command-decompose.md                ← oovra decompose (default, --full, --format=json)
│   ├── command-compare.md                  ← oovra compare (kind-aware, sequence-aware)
│   ├── errors.md                           ← every OovraError variant with trigger + canonical message
│   └── build-guide.md                      ← from-first-principles Rust construction guide
│
├── demos/                                  ← end-to-end demonstrations
│   ├── README.md                           ← demos index
│   ├── 01-lossless-roundtrip/              ← proof: compound alone is a self-extracting archive
│   ├── 02-text-prose-output/               ← the two external presentations of one file
│   ├── 03-deep-text-flattening/            ← --text recurses through every level
│   ├── 04-structural-diff/                 ← compare cuts through surface byte noise
│   └── 05-mixed-order-regression/          ← the body_level escalation fix (now a regression test)
│
└── version-reports/
    └── v0.1/                               ← v0.1-era planning artifacts
        ├── README.md                       ← v0.1 reports index
        ├── README2.md                      ← detailed report index (kept as a duplicate after folder merge)
        ├── SPEC-v0.2.md                    ← the v0.2 migration specification (Draft 3)
        ├── v0.2-scoping.md                 ← original scope contract for v0.2
        ├── v0.1-efficacy-rating.md         ← honest assessment of v0.1 as a shipped tool
        ├── v0.1-ceiling-analysis.md        ← structural limits and high-leverage adds
        └── oovra-server-feasibility.md     ← design exercise for a network service
```

All cross-links between these files have been verified to resolve. The reference, demos, and build guide describe the **current (v0.2)** state of the tool; the `version-reports/v0.1/` folder holds **historical** planning artifacts.
