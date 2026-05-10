# Oovra

A Rust tool for the **composition** and **comparison** of agentic system prompts from Markdown+TOML "prompt elements."

>
> *An **œuvre** is a body of work — the collected output of a maker. Oovra (the phonetic spelling, used in code and CLI) treats system prompts as composed works: assembled from named, versioned prompt elements that form your personal corpus.*

## What it does

Oovra is built around **one file format** — Markdown with TOML frontmatter — and **one numeric discriminator**: an `order` field.

- An **order-0 element** is atomic — a hand-authored sentence or paragraph that holds together on its own.
- An **order-1 element** is a composition of order-0 elements, produced by `oovra compose`.
- An **order-N element** is what you get when you compose elements that include at least two peers at order N−1. Order encodes compositional depth.

Every composed element is **self-describing and losslessly decomposable**: the body of an order-N element contains the complete files of its inputs (each itself an Oovra file), wrapped in chiral order-aware delimiters. `oovra decompose --full` recursively splits all the way down to order-0 leaves with full metadata intact — even on a file you just downloaded with no library context.

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
# Author an order-0 element
oovra create --new role-declaration --library ./elements --name "Role Declaration"

# Compose three order-0 elements into an order-1
oovra compose --library ./elements \
  --out-id coding-agent --out-name "Coding Agent" \
  -o ./elements/coding-agent.md \
  role-declaration refusal-policy-strict tone-direct

# Get a clean prompt to paste into a model (no Oovra metadata)
oovra compose --library ./elements --text \
  role-declaration refusal-policy-strict tone-direct

# Decompose one level down — list the immediate inputs
oovra decompose ./elements/coding-agent.md

# Decompose all the way to order-0 leaves as a folder tree
oovra decompose --full -o ./out ./elements/coding-agent.md

# Diff two compositions structurally (added/removed/version-changed inputs)
oovra compare ./elements/coding-agent-v1.md ./elements/coding-agent-v2.md
```

## The four operators

| Operator | What it does | Sheet analog |
|---|---|---|
| `oovra create` | Author a new order-0 element (or label an existing `.md`) | (cell entry) |
| `oovra compose` | Join ordered inputs into a higher-order element | JOIN |
| `oovra decompose` | Recover the inputs of a composed element (one level or `--full`) | SPLIT |
| `oovra compare` | Diff two elements (kind-aware: content for order-0, structural for order ≥1) | FORWARD-DIFF |

## File format

Every Oovra file is one `.md` with this exact shape:

```toml
+++
name = "Human-Readable Name"
order = 0
id = "kebab-case-id"
version = "1.0.0"
meta = "optional description; may be empty"
+++

(body — freeform Markdown for order-0;
 wrapped sub-element files for order ≥ 1)
```

For composed elements, the header also has `generated_at` (RFC 3339), `render_mode` (e.g. `"markdown-h2"`), `body_level` (an integer ≥ 1 governing the body delimiter), and `composed_of` (an array of `{id, version}` immediate-input records).

The body delimiter for a level-`N` composed element is `(N+1)`-tildes + `>>` to open and `(N+1)`-tildes + `<<` to close. So `body_level = 1` uses `~~>>` / `~~<<`, `body_level = 2` uses `~~~>>` / `~~~<<`, etc. The level is `max(input.order) + 1`, which always escalates strictly above any input's own body delimiter — chiral, monotonic, never collides with normal Markdown.

See [SCHEMA.md](./SCHEMA.md) for the full schema.

## Examples

[`oovra-by-example/`](./oovra-by-example/) is a curated suite of five end-to-end demos showing the tool in action. Each demo has a `notes.md` (what's being demonstrated and why it matters architecturally) and a `results.md` (captured outputs and verdicts) — together they cover lossless decomposition, the prose-form renderer, deep recursive flattening, structural diff, and the mixed-order delimiter-escalation case. Browse [`oovra-by-example/README.md`](./oovra-by-example/README.md) for the index.

## Why TOML, not YAML or JSON

Three reasons, in order:

1. **`composed_of = [{id="x", version="1.0.0"}, ...]` is what TOML inline tables were invented for.** This is the central data structure in the system, and TOML expresses it more naturally than YAML or JSON.
2. **Strict, no-coercion typing matters for self-describing round-trips.** YAML's implicit coercions (`version: 1.0` → float, `tags: [no, yes]` → bools) silently corrupt a system that has to losslessly recover what it wrote. TOML never coerces.
3. **The `+++` delimiter is a parser-disambiguator.** A tool can pick the right frontmatter parser from line 1 without peeking inside.

YAML's only meaningful win — Obsidian's Properties UI — is small relative to the type-safety wins. Body wikilinks, graph view, and search still work in Obsidian regardless of frontmatter format.

## Obsidian compatibility

Every Oovra file is a valid Markdown file, so the entire library is also a valid Obsidian vault. Open the `elements/` directory as a vault: bodies render as Markdown, `[[wiki-links]]` in bodies become live navigation, the graph view shows your prompt architecture. Oovra ignores wiki-links in bodies — they're a free property of the format, not a feature.

## Status

v0.1 — working. 32 tests passing (21 unit + 11 integration), including regression tests for the mixed-order body-delimiter collision case and the create-with-invalid-id orphan-file case. See [`oovra-build-guide.md`](./oovra-build-guide.md) for the from-first-principles step-by-step build guide derived from this codebase.

## License

Dual MIT / Apache-2.0 — see LICENSE.
