<p align="center">
  <img src="oovra_slate.png" alt=oovra_slate_logo Logo width=50% height =50% />
</p>
<h1 align="center">Oovra</h1> </p>

A Rust based, Obsidian compatible tool for the **composition** and **comparison** of agentic system prompts from Markdown+TOML "prompt elements."
>
> *An **œuvre** is a body of work — the collected output of a maker. Oovra (the phonetic spelling, used in code and CLI) treats system prompts as composed works: assembled from named, versioned prompt elements that form your personal corpus.*

## What it does

Oovra is built around **one file format** — Markdown with TOML frontmatter — and **one discriminator**: a `kind` field with exactly two values.

- A **`kind = "atom"`** element is hand-authored — a sentence or paragraph representing a primitive prompt element, such as a basic rule, instruction, policy, or context-providing statement.
- A **`kind = "compound"`** element is a composition produced by `oovra compose`. It carries a `composed_of` recipe plus composition metadata (`generated_at`, `render_mode`, `body_level`, `depth`).

Every compound is **self-describing and losslessly decomposable**: its body contains the complete files of its inputs (each itself an Oovra file), wrapped in chiral level-aware delimiters that strictly escalate with composition depth. `oovra decompose --full` recursively splits all the way down to atom leaves with full metadata intact — even on a file you just downloaded with no library context.

## Install / build

The build works identically on Windows, Linux, and macOS — same commands, same result. You need the [Rust toolchain](https://rustup.rs/) and a C linker.

### Prerequisites

| Platform | C linker setup |
|---|---|
| **Windows** | rustup installs the MSVC linker automatically when prompted; otherwise install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/?q=build+tools) |
| **Linux** (Debian/Ubuntu/WSL) | `sudo apt install build-essential pkg-config` |
| **Linux** (Fedora/RHEL) | `sudo dnf groupinstall "Development Tools"` |
| **Linux** (Arch) | `sudo pacman -S base-devel` |
| **macOS** | `xcode-select --install` |

### Install globally (recommended)

```bash
git clone https://github.com/crussella0129/oovra.git
cd oovra
cargo install --path .
# compiles in release mode and copies the binary to ~/.cargo/bin/oovra(.exe),
# which rustup put on your PATH — invoke `oovra` from any directory afterward
```

If `oovra` isn't found right after install, your PATH may need refreshing — open a new terminal, or on Linux/macOS run `source ~/.cargo/env`.

### Local build alternative

If you don't want a global install, build into the project's `target/` directory and run from there:

```bash
cargo build --release
./target/release/oovra --version        # Linux / macOS / WSL / git-bash
.\target\release\oovra.exe --version    # Windows PowerShell or cmd
```

## Quick tour

```bash
# Author an atom
oovra create --new role-declaration --library ./elements --name "Role Declaration"

# Compose three atoms into a compound
oovra compose --library ./elements \
  --out-id coding-agent --out-name "Coding Agent" \
  -o ./elements/coding-agent.md \
  role-declaration refusal-policy-strict tone-direct

# Get a clean prompt to paste into a model (no Oovra metadata)
oovra compose --library ./elements --text \
  role-declaration refusal-policy-strict tone-direct

# Decompose one level down — list the immediate inputs
oovra decompose ./elements/coding-agent.md

# Decompose all the way to atom leaves as a folder tree
oovra decompose --full -o ./out ./elements/coding-agent.md

# Diff two compositions structurally (added/removed/version-changed/moved inputs)
oovra compare ./elements/coding-agent-v1.md ./elements/coding-agent-v2.md

# Migrate a v0.1 library to v0.2 schema in place
oovra migrate ./elements
```

## The four operators (and one migration helper)

| Operator | What it does | Sheet analog |
|---|---|---|
| `oovra create` | Author a new atom (or label an existing `.md`) | (cell entry) |
| `oovra compose` | Join ordered inputs into a compound | JOIN |
| `oovra decompose` | Recover the inputs of a compound (one level or `--full`) | SPLIT |
| `oovra compare` | Diff two elements (kind-aware: content for atoms, **sequence-aware structural** for compounds) | FORWARD-DIFF |
| `oovra migrate` | Walk a v0.1 library and rewrite every file in v0.2 schema, in place | — |

`oovra compare` on two compounds reports four axes — `added`, `removed`, `version_changed`, and `moved` (the last new in v0.2). The axes are not mutually exclusive: a single input that has both been version-bumped *and* moved appears on both lists.

## File format

Every Oovra file is one `.md` with this exact shape:

```toml
+++
name = "Human-Readable Name"
kind = "atom"
id = "kebab-case-id"
version = "1.0.0"
meta = "optional description; may be empty"
+++

(body — freeform Markdown for atoms;
 wrapped sub-element files for compounds)
```

Compounds also carry `generated_at` (RFC 3339), `render_mode` (e.g. `"markdown-h2"`), `body_level` (an integer ≥ 1 governing the body delimiter), `depth` (the human-readable compositional-depth label, numerically equal to `body_level`), and `composed_of` (an array of `{id, version}` immediate-input records).

The body delimiter for a `body_level = N` compound is `(N+1)`-tildes + `>>` to open and `(N+1)`-tildes + `<<` to close. So `body_level = 1` uses `~~>>` / `~~<<`, `body_level = 2` uses `~~~>>` / `~~~<<`, etc. The level is always `max(input.body_level) + 1`, which strictly escalates above any input's own body delimiter — chiral, monotonic, never collides with normal Markdown.

See [`Documentation/reference/schema.md`](./Documentation/reference/schema.md) for the canonical schema reference.

## Documentation

The full user-facing documentation lives at [`Documentation/`](./Documentation/). Start at [`Documentation/README.md`](./Documentation/README.md) for the navigation hub. Highlights:

- **[`Documentation/reference/`](./Documentation/reference/)** — feature-by-feature specifications: every command, every flag, every error variant, every schema field.
- **[`Documentation/demos/`](./Documentation/demos/)** — end-to-end demos with captured outputs and verdicts, split by version (`v0.1/` covers lossless round-trip, prose rendering, deep flattening, structural diff, mixed-order regression; `v0.2/` covers the full operator gamut with sequence-aware compare).
- **[`Documentation/reference/build-guide.md`](./Documentation/reference/build-guide.md)** — the from-first-principles construction guide for anyone reimplementing this from scratch.
- **[`CHANGELOG.md`](./CHANGELOG.md)** — v0.1 → v0.2 change list.

## Why TOML, not YAML or JSON

Three reasons, in order:

1. **`composed_of = [{id="x", version="1.0.0"}, ...]` is what TOML inline tables were invented for.** This is the central data structure in the system, and TOML expresses it more naturally than YAML or JSON.
2. **Strict, no-coercion typing matters for self-describing round-trips.** YAML's implicit coercions (`version: 1.0` → float, `tags: [no, yes]` → bools) silently corrupt a system that has to losslessly recover what it wrote. TOML never coerces.
3. **The `+++` delimiter is a parser-disambiguator.** A tool can pick the right frontmatter parser from line 1 without peeking inside.

YAML's only meaningful win — Obsidian's Properties UI — is small relative to the type-safety wins. Body wikilinks, graph view, and search still work in Obsidian regardless of frontmatter format.

## Obsidian compatibility

Every Oovra file is a valid Markdown file, so the entire library is also a valid Obsidian vault. Open the `elements/` directory as a vault: bodies render as Markdown, `[[wiki-links]]` in bodies become live navigation, the graph view shows your prompt architecture. Oovra ignores wiki-links in bodies — they're a free property of the format, not a feature.

## Status

**v0.2 — shipped.** 46 tests passing (27 unit + 19 integration), including regression tests for the mixed-input body-delimiter collision case, the create-with-invalid-id orphan-file case, and the recursive frontmatter-rewrite case caught during the v0.1→v0.2 migration tooling work. See [`Documentation/reference/build-guide.md`](./Documentation/reference/build-guide.md) for the from-first-principles step-by-step build guide derived from this codebase.

v0.1 documents and planning artifacts live under [`Documentation/version-reports/v0.1/`](./Documentation/version-reports/v0.1/) as the historical record.

## License

Dual-licensed under either of:

- MIT License ([LICENSE-MIT](./LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
