# Oovra by Example

This directory is the **complete user-facing documentation snapshot** for Oovra v0.1. It's split into two parts:

- **[`reference/`](./reference/)** — feature-by-feature specifications. Every command, every flag, every error variant, every schema field. Look here when you need to know *how* something works.
- **[`demos/`](./demos/)** — end-to-end demonstrations of the tool in action. Each demo is a focused walk-through of one architectural property, with input files, captured outputs, and verdicts. Look here to see *that* something works.

Think of this folder as Oovra's `docs/` directory. It is a snapshot — captured against the current source code and intended to be updated as the tool evolves. The git history of `oovra-by-example/` tracks what changed across versions.

---

## I want to learn Oovra — where do I start?

If this is your first time:

1. **Read [`demos/01-lossless-roundtrip/notes.md`](./demos/01-lossless-roundtrip/notes.md) and its [`results.md`](./demos/01-lossless-roundtrip/results.md)** — concrete proof of the system's central claim (a composed file is a self-extracting archive). 5 minutes.
2. **Read [`demos/02-text-prose-output/notes.md`](./demos/02-text-prose-output/notes.md)** — see the two presentations (on-disk vs paste-ready). 5 minutes.
3. **Skim [`reference/schema.md`](./reference/schema.md)** — the file format in detail. 10 minutes.
4. **Skim [`reference/command-compose.md`](./reference/command-compose.md)** — the central operation. 10 minutes.
5. **Pick the rest as needed** — demos 3–5 for deeper architectural claims; reference docs for command-by-command lookups.

If you're already familiar with the tool and need to look something up: jump directly to the relevant doc in [`reference/`](./reference/) via the index in [`reference/README.md`](./reference/README.md).

---

## I want to use Oovra — what's the workflow?

A typical session, with links to the relevant docs:

```bash
# 1. Author atomic elements (one-liner per element)
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

For builders and extenders, the relevant docs are in the parent directory:

- **[../SCHEMA.md](../SCHEMA.md)** — the terse, agent-facing schema contract
- **[../oovra-build-guide.md](../oovra-build-guide.md)** — the from-first-principles construction guide (full Rust walkthrough)

The `oovra-by-example/` folder you're reading now is the user-facing complement to those. It assumes the tool exists and helps you use it; the build guide assumes nothing and helps you reproduce it.

---

## Snapshot status

This documentation snapshot was generated against **Oovra v0.1.0**. The 5 demos were captured at commit time; their results files contain actual SHA256 hashes, file sizes, line counts, and structural-diff outputs from real runs.

When the source code changes (new commands, new schema fields, new render modes, etc.), the corresponding docs in `reference/` will be updated and the demos in `demos/` will be re-run. Treat any file in here as a snapshot of the version it was generated against — see the commit history for the dated lineage.

---

## Structure at a glance

```
oovra-by-example/
├── README.md                          ← you are here
│
├── reference/
│   ├── README.md                      ← reference index + reading order
│   ├── schema.md                      ← TOML schema spec: every field, every constraint, every invariant
│   ├── order-and-delimiters.md        ← the two-integer system (order, body_level) + the chiral delimiter scheme
│   ├── command-create.md              ← oovra create (--new and --label modes, every flag)
│   ├── command-compose.md             ← oovra compose (file, --text, --re-render modes)
│   ├── command-decompose.md           ← oovra decompose (default, --full, --format=json)
│   ├── command-compare.md             ← oovra compare (4-way dispatch table)
│   └── errors.md                      ← every OovraError variant with example trigger + canonical message
│
└── demos/
    ├── README.md                      ← demos index
    ├── 01-lossless-roundtrip/         ← proof: composed file alone is a self-extracting archive
    ├── 02-text-prose-output/          ← the two external presentations of one file
    ├── 03-deep-text-flattening/       ← --text recurses through every level
    ├── 04-structural-diff/            ← compare cuts through surface noise
    └── 05-mixed-order-regression/     ← the body_level escalation fix (where order != body_level)
```

Every file has explicit cross-links to related docs. The reference and demos reinforce each other — refer back-and-forth as you read.
