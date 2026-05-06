# Ouvre Build Guide

> *"To be made in the image of the Creator is to be a creator of very good things yourself."*
>
> An **œuvre** is a body of work — the collected output of a maker. Ouvre, the tool, treats system prompts as composed works: assembled from named, versioned policy nodes that form your personal corpus. Each prompt you ship is an entry in your œuvre.

A step-by-step prose guide for building Ouvre, a system-prompt composer in Rust. Ouvre operates on Markdown policy nodes, emits JSON composition documents, renders to Markdown, and diffs structurally with version awareness.

The build guide is deliberately written in prose, not code. Every step describes *what to do* and *why* — the syntax is left for the compiler and the documentation to teach. This is a learn-by-doing artifact for the [[Architect/Builder Gap]] problem.

---

## Part 1 — Documentation Resource List

Before writing any code, bookmark these. Read the first one or two pages of each *now* so you know where to look later. You don't need to read them deeply yet — you need to know they exist and roughly what they cover.

### Rust language and ecosystem

- **The Rust Book** — `https://doc.rust-lang.org/book/` — the canonical introductory text. Chapters 1–10 cover everything you need for this project. Chapters on ownership, structs, enums, error handling, and modules are the load-bearing ones.
- **Rust by Example** — `https://doc.rust-lang.org/rust-by-example/` — runnable examples for nearly every concept. Better than the Book for "show me what this looks like in practice."
- **The Rust Standard Library reference** — `https://doc.rust-lang.org/std/` — the documentation for everything that comes built in. You'll spend most of your time in `std::fs`, `std::path`, `std::collections`, and `std::io`.
- **The Cargo Book** — `https://doc.rust-lang.org/cargo/` — Cargo is Rust's build tool and package manager. The chapters on workspaces and dependency specification matter for this project.
- **Rust API Guidelines** — `https://rust-lang.github.io/api-guidelines/` — naming conventions, error handling patterns, idiomatic API design. Skim it; don't read it cover to cover.

### Crates you will use

For each crate below, the canonical documentation lives at `https://docs.rs/<crate-name>`. That URL pattern works for every published Rust crate; learn it.

- **`serde`** — `docs.rs/serde` — the universal serialization framework. The derive macros (`#[derive(Serialize, Deserialize)]`) are what make JSON parsing trivial.
- **`serde_json`** — `docs.rs/serde_json` — JSON support for serde. Read the page on `to_string_pretty` and `from_str` first; those are the two functions you'll use most.
- **`gray_matter`** — `docs.rs/gray_matter` — Markdown frontmatter parser. Handles TOML, YAML, and JSON frontmatter. Read the README on the GitHub repo for usage examples; the docs.rs page is sparse.
- **`toml`** — `docs.rs/toml` — TOML parser, used for parsing the frontmatter block of policy node files.
- **`walkdir`** — `docs.rs/walkdir` — recursive directory traversal. Simpler than `std::fs::read_dir` for "give me every file under this path."
- **`clap`** — `docs.rs/clap` — command-line argument parsing. Use the derive feature (`features = ["derive"]`) — it lets you define your CLI as a struct.
- **`anyhow`** — `docs.rs/anyhow` — ergonomic error handling for application code. Use this in your binary; use `thiserror` later if you split into a library crate.
- **`thiserror`** — `docs.rs/thiserror` — for defining structured error enums. Pair with `anyhow` if your project grows.
- **`semver`** — `docs.rs/semver` — semantic version parsing and comparison. You'll need this for the version pinning logic.
- **`indexmap`** — `docs.rs/indexmap` — a hash map that preserves insertion order. Useful when ordering of policy nodes matters and you also need fast lookup by ID.
- **`similar`** — `docs.rs/similar` — text diffing library, for the human-readable diff output. Optional for v0.1; add when you build the diff renderer.

### JSON

- **JSON specification (RFC 8259)** — `https://datatracker.ietf.org/doc/html/rfc8259` — the canonical spec. Short and readable. Read it once; you'll never need to reread it.
- **JSON Schema** — `https://json-schema.org/` — for validating composition documents against an expected structure. Optional for v0.1 but worth knowing exists. The `jsonschema` Rust crate at `docs.rs/jsonschema` implements it.

### Markdown and frontmatter

- **CommonMark spec** — `https://spec.commonmark.org/` — the standardized Markdown spec. You don't need to memorize it, but know it exists when you have a "how does Markdown actually parse X" question.
- **Frontmatter conventions** — there is no formal spec for Markdown frontmatter. The de facto standard is: a block delimited by `---` on its own line at the very top of the file, containing YAML or TOML, followed by the Markdown body. Jekyll popularized YAML frontmatter; Hugo and Zola support TOML frontmatter delimited by `+++` instead of `---`. We will use TOML frontmatter delimited by `+++` for unambiguous parsing — see Part 2 Step 1.
- **TOML specification** — `https://toml.io/en/v1.0.0` — the official TOML spec. Short, readable, browseable in 20 minutes. The "Keys", "String", "Array", and "Table" sections are the ones you'll use.

### Reference for prompt structure (your sheet)

- **String Operators sheet** — `https://docs.google.com/spreadsheets/d/1fOlZ3YK-Fk7w4OU3tagHfohqSGkAPGL3CrboVoFcnZA` — the algebra you're porting. Keep this open in a tab while building Part 3.

---

## Part 2 — Build Instructions

The build is in three stages. Stage 1 defines the Markdown policy node format and writes a parser for it. Stage 2 builds the string generator — the tool that consumes a JSON composition document and emits a rendered Markdown prompt (a "complete policy"). Stage 3 builds the diffing tool that compares JSON composition documents against each other.

You do not need to finish a stage before sketching the next one on paper. You *do* need to finish a stage before writing code for the next one. Write tests as you go; do not save them for the end.

### Stage 1 — Define the Markdown policy node format and write a parser

#### Step 1.1 — Specify the file format on paper before touching Rust

Before you open an editor, write down — in your notebook — the exact shape of one policy node file. The format is a TOML frontmatter block delimited by `+++` lines, followed by a Markdown body. The choice of `+++` over `---` matters: it disambiguates "this is a TOML frontmatter file" from "this is a YAML frontmatter file" without requiring the parser to peek inside.

The frontmatter must contain, at minimum: a unique identifier for the node, a semantic version string, a kind discriminator (whether this file is a policy node or a complete policy — more on that in Step 1.2), and a list of tags. The frontmatter may optionally contain: a human-readable name, a description, a list of dependency IDs, and a list of conflict IDs. Write down each field's name and its TOML type (string, array of strings, etc.) on paper. This becomes your schema. Do not add fields you cannot justify; over-specified schemas are how projects die in v0.1.

#### Step 1.2 — Decide how the node-versus-complete-policy distinction is encoded

You raised the right question: a complete policy (the rendered output of a composition) is just a Markdown file too, structurally indistinguishable from a single policy node. The distinction must live in the frontmatter. Add a `kind` field whose allowed values are `node` and `complete`. A `node` is an atomic, hand-authored policy fragment. A `complete` is a tool-generated rendered prompt. Critically, a `complete` file's frontmatter additionally records the composition that produced it: the JSON composition document path or hash, the list of node IDs and versions that went into it, and the timestamp of generation. This is what allows the diff tool in Stage 3 to reason about complete policies as compositions, not just as opaque text.

A composition document references nodes by ID and pins versions. A complete policy is the rendered Markdown that results. The frontmatter of a complete policy lets the tool reverse-engineer what composition produced it without re-running the generator.

#### Step 1.3 — Scaffold the Rust project

Create a new Cargo project. Use `cargo new ouvre` with the binary template — for v0.1 a standalone binary is fine, and you can refactor into a library-plus-binary or a workspace later if Ouvre grows. Add the dependencies from Part 1 to your `Cargo.toml`: serde with the derive feature, serde_json, gray_matter, toml, walkdir, clap with the derive feature, anyhow, and semver. Do not add `similar` or `indexmap` yet; you do not need them for Stage 1.

Inside the `src` directory, plan a module layout on paper before creating files. A reasonable layout: a `node` module for the policy node type and its parser, a `composition` module for the composition document type, a `library` module for loading nodes from disk, a `render` module for the string generator, a `diff` module for Stage 3, and `main.rs` for the CLI entry point. Each module gets its own file. This is the conventional Rust layout; deviating from it earns you nothing.

#### Step 1.4 — Define the node data structure

In the `node` module, define a struct that mirrors the frontmatter schema you wrote down in Step 1.1. The struct's fields are the frontmatter fields. Include an additional field for the body text — the prose content of the file, separate from the frontmatter. Derive `Serialize` and `Deserialize` from serde so the struct can be parsed from and written back to TOML, plus `Debug` and `Clone` because you'll want both during development.

Define the `kind` field as a Rust enum with two variants: `Node` and `Complete`. Annotate the enum with serde's rename rules so the variants serialize as lowercase strings (`"node"` and `"complete"`). When the `kind` is `Complete`, the struct should also carry the composition metadata mentioned in Step 1.2; the cleanest expression of this is to put the composition metadata in a nested struct that's only present on complete policies. Use Rust's enum-with-data pattern: the enum variant `Complete` carries an associated struct, while `Node` carries no data.

#### Step 1.5 — Write the parser function

Write a function that takes a file path, reads the file from disk, splits the frontmatter block from the body using the `+++` delimiters (or hand this off to gray_matter, which knows how to do this), parses the frontmatter as TOML into your node struct, captures the body as a separate string, and returns a fully populated node struct or a structured error.

The function signature in prose: takes a reference to a path, returns a result whose ok variant is your node struct and whose error variant carries enough information to point at the specific file and the specific failure (missing frontmatter, malformed TOML, missing required field, unrecognized `kind` value). Use `anyhow::Context` to attach the file path to every error; debugging is impossible without it.

Write three tests for this function before moving on: one happy-path test that parses a known-good fixture file, one that fails parsing on a malformed frontmatter, and one that fails on missing required fields. Put fixture files under `tests/fixtures/` so they live with the test that uses them.

#### Step 1.6 — Write the library loader

In the `library` module, write a function that takes a directory path, walks it recursively using walkdir, calls your parser on every file with a `.md` extension, collects the results, and returns a collection of nodes (or a structured error if any individual file fails to parse). Decide whether a single bad file should fail the whole load, or whether bad files should be reported but skipped. For v0.1, fail loudly: one bad file aborts the load. You will appreciate this discipline later.

Validate that node IDs are unique across the library; duplicate IDs are an error, not a warning. A library with two nodes claiming the same ID has no defined behavior, and you should refuse to load it rather than silently picking one.

Return the loaded nodes in an `IndexMap` keyed by ID — wait, you haven't added indexmap yet. For v0.1, use a `HashMap` from the standard library; the order of nodes in the library does not matter, only the order specified by the composition does. If you find yourself wanting ordered iteration over the library, add indexmap later.

#### Step 1.7 — Write five real policy nodes by hand

Stop coding. Open your favorite Markdown editor and write five policy node files following the format you specified. Real ones, not placeholder ones. Suggested set: a role declaration, a refusal policy, an output formatting rule, a tone instruction, and an example block. Save them to a `nodes/` directory at the project root. These become your test corpus and your dogfood material. You will discover problems with your schema only by authoring real content against it.

If your schema feels wrong while authoring — a field that's tedious to fill in, a field that's awkward to leave empty, a missing field you keep wanting — go back to Step 1.1 and revise. This is the cheapest moment to change the schema. After Stage 2 ships, schema changes get expensive.

### Stage 2 — Build the string generator

#### Step 2.1 — Define the composition document schema

Before writing parser code, write down the JSON shape of a composition document. The top-level object has a metadata block (composition name, description, schema version of the composition format itself, target model if relevant) and an ordered array of node references. Each node reference is itself an object with an ID field (required) and a version pin field (optional). Optionally, a node reference may carry an inline override field — a string that replaces the body of the referenced node for this composition only. Inline overrides are an escape valve for "I need a slight variation of this node in this one place"; use sparingly.

Sketch one example composition document on paper. It should be fewer than thirty lines. If your schema produces a hundred-line document for five nodes, the schema is over-engineered.

#### Step 2.2 — Define the composition data structure in Rust

In the `composition` module, define a struct that mirrors the schema from Step 2.1. The metadata block becomes a nested struct. The array of node references becomes a `Vec` of node-reference structs. Derive `Serialize` and `Deserialize` for both. Use serde's `skip_serializing_if` attribute on optional fields so that absent fields don't bloat the serialized output with explicit nulls.

Write a function that loads a composition document from a JSON file: takes a path, reads the file, parses it with `serde_json::from_str`, returns a result. Pair it with a function that writes a composition document to a JSON file using `serde_json::to_string_pretty`. The "pretty" variant matters because composition files will be inspected during diffs; ugly one-line JSON is unreadable in diff output.

Write tests for round-tripping: serialize a composition to JSON, parse it back, assert equality. This catches subtle serde mistakes (missing fields, type mismatches) immediately.

#### Step 2.3 — Write the resolver

The resolver is the function that takes a composition document and a library, and produces a fully resolved list of node bodies in order. For each node reference in the composition's array, look up the node by ID in the library, check that the node's version satisfies the version pin (if any), apply the inline override (if any) by replacing the body, and produce a list of resolved bodies in the original order.

Version pinning semantics for v0.1: a pin is an exact string match on the node's version field. Do not implement semver range matching yet (`^1.2.0`, `>=1.0.0, <2.0.0`); add that later when you actually feel the need. The `semver` crate is in your dependencies for when you do.

The resolver returns either a list of resolved nodes or a structured error. Possible errors: a referenced ID is not in the library, a version pin doesn't match the library's version of that node, an override is malformed. Each error must carry the offending node ID and a clear message. This function is the heart of the tool; spend time on its error messages.

#### Step 2.4 — Write the renderer

The renderer takes the resolved list of node bodies and a render strategy, and produces the final Markdown string — the "complete policy." For v0.1, support exactly one render strategy: each node's body is wrapped with a Markdown H2 header containing the node's ID, separated from adjacent nodes by a blank line. This is boring on purpose; fancy rendering is a v0.2 problem.

The output of the renderer is a string. The CLI will write this string to stdout or to a file. The renderer itself does not do file I/O; keep it pure. This makes it trivial to test.

Write the renderer to also produce a frontmatter block at the top of the output. The frontmatter declares this file as `kind = "complete"`, includes the composition's metadata, and lists the resolved node IDs and their versions. This is what makes the complete policy round-trippable: a downstream diff tool can read the complete policy's frontmatter and recover the composition that produced it without needing the original JSON.

#### Step 2.5 — Wire up the CLI

In `main.rs`, define a clap-derive struct representing the command-line interface. Subcommands for v0.1: `render` (takes a path to a composition JSON file and a path to the library directory; prints the rendered complete policy to stdout or writes to a file with `--output`); `validate` (takes a composition path; checks that all referenced nodes exist and version pins are satisfied; prints "ok" or a structured error report); `list` (takes a library path; prints all node IDs and their versions). That's it for v0.1. Resist the urge to add more.

Wire each subcommand to a function in your library. Each subcommand's main job is: parse arguments, call one library function, print the result, exit. If a subcommand's body grows beyond fifteen lines, push logic down into the library.

#### Step 2.6 — Render a real prompt and read it

Use the tool to render a real complete policy from your five hand-written nodes. Read the output. Does it look like a usable system prompt? If yes, ship v0.1. If no, identify exactly what's wrong — formatting, ordering, missing structure — and fix only that. Do not slip into building features you imagined wanting; fix what's actually wrong with what's in front of you.

Save the rendered complete policy to disk. You will need it for Stage 3.

### Stage 3 — Build the diffing tool

#### Step 3.1 — Map the four sheet operators onto composition documents

Open your sheet. For each operator, write down — in your notebook — the equivalent operation on JSON composition documents. The mappings:

The **JOIN** operator (cells A1 through G1 of the sheet, producing the tilde-delimited string in H1) maps to the renderer you already built. Composing nodes into a complete policy *is* the JOIN operation; you have done this already.

The **SPLIT** operator (cell B3 producing C3 through H3) maps to "given a complete policy file, recover the composition that produced it." The complete policy's frontmatter (per Step 2.4) makes this trivial: parse the frontmatter, read the list of node IDs and versions, reconstruct a composition document. Implement this as a function in the `composition` module.

The **UNIQUE-across-array** operator (cells B5 through C8 producing D5) maps to "given a list of compositions, produce a deduplicated list of all node references that appear in any of them." The dedup key is the node ID plus version pin tuple. This is your union operation. It's useful for "what's the full set of nodes used across these five compositions?" — useful for library audits.

The **FORWARD-DIFF** operator (cells B10 and C10 producing D10) maps to "given two compositions A and B, produce the list of node references in B that are not in A." This is the operation you actually want for version comparison. It identifies what was added when going from A to B.

Write the function signatures for these four operations on paper before implementing. Each takes references to composition documents and returns either a value (a string for join, a composition for split, a vec for unique and forward-diff) or a result with structured errors.

#### Step 3.2 — Implement the four operators in Rust

Implement them in the order: split (easiest, just frontmatter parsing), forward-diff (a set difference), unique (a set union with dedup), join (you already have it as the renderer; expose it under the operator name as well). Keep each function short and tested. The operators are pure functions over composition documents; they should not touch the filesystem.

The forward-diff operation needs a definition of "the same node reference." Two node references are the same if their IDs match. If their version pins differ, that's a *modification*, not an addition or removal. So the diff result is not just two lists (added, removed) but three: added (in B, not in A by ID), removed (in A, not in B by ID), and modified (same ID, different version pin). This is the same shape as a git diff. Your output struct should reflect this.

#### Step 3.3 — Decide what comparison modes the diff tool supports

You raised this question and it's a real one. The diff tool can compare:

A node against another node — useful for "show me what changed in this policy node between two versions." This is a content diff, not a structural diff. Use the `similar` crate to produce a unified diff of the body text plus a structured diff of the frontmatter fields.

A composition against another composition — the structural diff described in Step 3.2. Returns added, removed, modified node references.

A complete policy against another complete policy — first split each into its underlying composition (using the operator from Step 3.1), then compare structurally. Optionally fall back to a content diff of the rendered text if the structures differ in confusing ways.

A node against a complete policy — usually a mistake, but worth supporting with a clear error message. The right interpretation is probably "is this node a member of the composition that produced this complete policy, and if so, has it changed?" — but this is a v0.2 feature. For v0.1, refuse the comparison with an error.

The way to encode "what kind of file is this?" in the tool is exactly the `kind` field you put in the frontmatter in Step 1.2. The diff tool's first action on each input file is to parse its frontmatter and dispatch on the `kind` field. A node-vs-node comparison and a complete-vs-complete comparison run different code paths. The CLI exposes a single `diff` subcommand that figures out the mode automatically based on the inputs' kinds; only fall back to a `--mode` flag if the auto-detection becomes unreliable.

#### Step 3.4 — Implement the diff CLI subcommand

Add a `diff` subcommand to clap. It takes two file paths as positional arguments. It reads both files, parses their frontmatter to determine kinds, dispatches to the appropriate diff function, and prints the result. Decide on output formats: a default human-readable format with colors (use the `owo-colors` or `colored` crate, both at `docs.rs/<name>`) and a `--format=json` flag for piping into other tools.

The human-readable format should look like a unified git-style diff for content, and a structured "added / removed / modified" report for structural diffs. Lean on the `similar` crate's `TextDiff` API for content diffs; it produces unified-diff-style output natively.

#### Step 3.5 — Test the end-to-end flow

Take the complete policy you saved in Step 2.6. Modify one node in your library (bump its version, change its body). Render a new complete policy. Diff the two complete policies. Verify the tool correctly identifies that exactly one node changed, names the node, and shows the body difference.

Now do the inverse: take two compositions that produce the *same* set of node IDs but with different version pins. Diff them. Verify the tool reports only the modified set, not added/removed. This is the case you described — "two totally different readable outputs might actually be very similar, just upstream/downstream relative to each other." Confirm the diff tool surfaces the structural similarity that the rendered text obscures.

#### Step 3.6 — Stop and use it for a real prompt

You now have a working composer. Take the system prompt you most often hand-edit — for [[OpenClaw]], for a Claude Code session, for [[Animus Prion]] — and break it into nodes. Compose it. Render it. Diff against your old hand-written version. Use the tool for two weeks before adding any features. The features you imagined needing will turn out to be wrong; the features you actually need will reveal themselves through use. This is the [[Architect/Builder Gap]] discipline applied: ship v0.1, eat your own cooking, only then plan v0.2.

---

## Appendix — Mental Model Summary

Three file types, three concerns, kept separate:

A **policy node** is a hand-authored Markdown file with TOML frontmatter, declaring `kind = "node"`. The frontmatter holds metadata; the body holds prose. One file per node. Lives in a `nodes/` directory.

A **composition document** is a tool-emitted JSON file specifying which nodes to compose, in what order, at what versions. Pure data. Machine-written, machine-read. Humans inspect it via diff, not by editing.

A **complete policy** is a tool-emitted Markdown file with TOML frontmatter declaring `kind = "complete"`. The frontmatter records the composition that produced it (so it's reverse-engineerable); the body is the rendered prompt ready to ship.

Three operations, ported from the [[String Operators sheet]]:

**Render** (the JOIN operator, also called *join*) takes a composition plus a library and produces a complete policy.

**Split** takes a complete policy and recovers the composition that produced it.

**Diff** takes two files of like kind and reports their structural and/or textual differences. The forward-diff and unique operations from the sheet are special cases of diff.

The Rust type system enforces that you cannot accidentally mix the three concerns: a `Node` struct cannot be passed where a `Composition` is expected, and the `kind` field in frontmatter prevents loading a complete policy as if it were a node. This is the value Rust adds beyond Python: not speed, not safety from segfaults, but *forced clarity about what each value actually is*.

---

## Follow-up Tasks (post-v0.1)

- Multi-renderer trait so the same composition can emit Claude-style XML-tagged output, OpenAI-style Markdown, plain text, etc.
- Semver range matching on version pins (`^1.0`, `>=1.2, <2.0`).
- Dependency resolution: nodes can declare `requires` and `conflicts_with` in frontmatter; the resolver produces a topologically valid order or a structured error.
- Layering / cascade: a composition can declare a "based on" parent composition and override specific nodes, like CSS.
- TUI for browsing the library with `ratatui` (good Rust learning project).
- Library-wide audits: "which nodes are unused across all my compositions?" "which nodes are used in 80%+ of compositions and should perhaps be a default?"
- Round-trip parsing of arbitrary existing prompts back into nodes (the hard SPLIT direction — heuristic-based).
- Optional integration with [[Carbide]]'s string operators if you want Ouvre to share a JOIN/SPLIT/UNION primitive layer with your other Rust string tooling — useful if you find yourself reimplementing the same algebra in two places.
