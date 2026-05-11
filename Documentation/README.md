# Documentation

The complete user-facing and builder-facing documentation snapshot for Oovra v0.1. Three sibling sections, each serving a different audience:

| Section                                 | For                                                  | Style                                                                                                |
|-----------------------------------------|------------------------------------------------------|------------------------------------------------------------------------------------------------------|
| [`reference/`](./reference/)            | Users looking up "how do I do X?"                    | Feature-by-feature specifications — every command, every flag, every error variant, every schema field |
| [`demos/`](./demos/)                    | Users wanting to see "does this actually work?"      | End-to-end walkthroughs with concrete inputs, captured outputs, byte-equality verdicts                |
| [`build-guide.md`](./build-guide.md)    | Builders/forkers wanting to "reimplement this"       | From-first-principles construction guide; the full Rust walkthrough                                   |

The terse machine-facing schema contract is one level up at [`../SCHEMA.md`](../SCHEMA.md). It's at repo root rather than inside `Documentation/` because conventionally, contracts (the public API surface) live where consumers expect to find them.

---

## I want to learn Oovra — where do I start?

The shortest path to understanding what the tool does:

1. **[`demos/01-lossless-roundtrip/notes.md`](./demos/01-lossless-roundtrip/notes.md)** and its [`results.md`](./demos/01-lossless-roundtrip/results.md) — concrete proof of the system's central claim that a composed file is a self-extracting archive. ~5 minutes.
2. **[`demos/02-text-prose-output/notes.md`](./demos/02-text-prose-output/notes.md)** — see the two external presentations of a single Oovra file. ~5 minutes.
3. **[`reference/schema.md`](./reference/schema.md)** — the file format in detail. ~10 minutes.
4. **[`reference/command-compose.md`](./reference/command-compose.md)** — the central operation. ~10 minutes.
5. **Pick the rest as needed** — demos 3–5 for deeper architectural claims; remaining reference docs for command-by-command lookups.

If you've used the tool already and need to look something up: skip directly to the relevant doc in [`reference/`](./reference/) — its [`README.md`](./reference/README.md) is the per-doc index.

---

## I want to use Oovra — what's the workflow?

A typical session, with links to the relevant docs:

```bash
# 1. Author atomic elements (one per element)
oovra create --new role-statement --library ./prompts --name "Role Statement"
# ... edit ./prompts/role-statement.md to add your prompt text

# 2. Compose them into a higher-order prompt
oovra compose --library ./prompts --out-id coding-agent --out-name "Coding Agent" \
  -o ./prompts/coding-agent.md \
  role-statement safety-fence tone-direct

# 3. Get a paste-ready prompt for your model
oovra compose --library ./prompts --text coding-agent
# → outputs clean Markdown with ## headers per element

# 4. After editing some atomics, see what changed structurally
oovra compare ./prompts-old/coding-agent.md ./prompts/coding-agent.md
# → shows added/removed/version-changed inputs

# 5. Recover the inputs of any composed file (even one you didn't compose yourself)
oovra decompose --full ./prompts/coding-agent.md -o ./out
# → writes the full input tree to ./out/coding-agent/
```

Each step is documented in detail in the corresponding `reference/command-*.md`.

---

## I want to extend or fork Oovra — what should I read?

For builders, the [`build-guide.md`](./build-guide.md) walks through the full Rust implementation from scratch: schema design decisions, parser internals, the serde derive patterns, the validator's joint invariant, the body-delimiter scheme, the dispatch in `compare`, the test strategy. It's written so that someone with basic Rust experience can reproduce the v0.1 tool by following it.

The build guide assumes nothing about prior familiarity with the tool — it's the bottom-up construction. Pair it with [`reference/schema.md`](./reference/schema.md) (which is the *contract* the build guide implements) and [`../SCHEMA.md`](../SCHEMA.md) (the terse agent-facing version of the same contract) for the full picture.

---

## Snapshot status

This documentation snapshot was generated against **Oovra v0.1.0**. The 5 demos were captured at commit time; their `results.md` files contain actual SHA256 hashes, file sizes, line counts, and structural-diff outputs from real runs.

When the source code changes (new commands, new schema fields, new render modes, etc.), the corresponding docs in `reference/` are updated and the demos in `demos/` are re-run. Treat any file in here as a snapshot of the version it was generated against — see git history for the dated lineage.

---

## Structure at a glance

```
Documentation/
├── README.md                          ← you are here (the docs navigation hub)
│
├── reference/                          ← feature-by-feature specifications
│   ├── README.md                       ← reference index + reading order
│   ├── schema.md                       ← TOML schema: every field, every constraint, every invariant
│   ├── order-and-delimiters.md         ← the two-integer system (order, body_level) + the chiral delimiter scheme
│   ├── command-create.md               ← oovra create (--new and --label modes, every flag)
│   ├── command-compose.md              ← oovra compose (file, --text, --re-render modes)
│   ├── command-decompose.md            ← oovra decompose (default, --full, --format=json)
│   ├── command-compare.md              ← oovra compare (4-way dispatch table)
│   └── errors.md                       ← every OovraError variant with trigger + canonical message
│
├── demos/                              ← end-to-end demonstrations
│   ├── README.md                       ← demos index
│   ├── 01-lossless-roundtrip/          ← proof: composed file alone is a self-extracting archive
│   ├── 02-text-prose-output/           ← the two external presentations of one file
│   ├── 03-deep-text-flattening/        ← --text recurses through every level
│   ├── 04-structural-diff/             ← compare cuts through surface byte noise
│   └── 05-mixed-order-regression/      ← the body_level escalation fix
│
├── build-guide.md                       ← from-first-principles Rust construction guide
│
└── v0.1/                                ← version-specific artifacts (planning, assessments)
    └── version reports/                 ← three reports written during v0.1
        ├── v0.2-scoping.md              ← scope contract for the v0.2 release
        ├── oovra-server-feasibility.md  ← design exercise for a network service
        └── v0.1-efficacy-rating.md      ← honest assessment of v0.1 as a shipped tool
```

All cross-links between these files have been verified to resolve. The reference, demos, and build guide describe the **current** state of the tool; the `v0.1/` folder holds **historical** planning artifacts that snapshot what was being considered when v0.1 was the shipping version.
