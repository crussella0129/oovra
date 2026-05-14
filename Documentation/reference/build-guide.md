+++
name = "Oovra Build Guide (v0.2, derived from working code)"
kind = "atom"
id = "oovra-build-guide"
version = "0.6.0"
meta = "Step-by-step from-first-principles guide for building Oovra in Rust, derived from the v0.2 codebase. Written for someone who has never written Rust."
+++

# Oovra Build Guide

> An **œuvre** is a body of work — the collected output of a maker. Oovra (the phonetic spelling) treats system prompts as composed works: assembled from named, versioned **prompt elements** that form your personal corpus. Each prompt you ship is an entry in your œuvre.

> **v0.2 migration note.** This guide reflects the v0.2 schema: every element is either an **atom** (`kind = "atom"`, hand-authored) or a **compound** (`kind = "compound"`, produced by `oovra compose`). The v0.1 numeric `order` field has been replaced by `kind`. If you have v0.1 files on disk, run `oovra migrate <library-dir>` to convert them in place — `migrate` is itself a Stage 5 operator described later in this guide.

This is a **learn-by-doing** guide for building Oovra in Rust. It is derived from a complete working v0.2 implementation — every pattern below is in the codebase, and every tradeoff was decided in the act of building. Where the original spec said "do X," this guide says "do X, **here is what that actually looks like in Rust**, and here is **why** the alternative was rejected."

The build is in **five stages**, mirroring the four operators of the Sheet algebra you're porting plus the v0.2 migration tool:

| Sheet operator | Oovra operator | Stage |
|---|---|---|
| (cell entry) | `oovra create` | Stage 2 |
| JOIN (`A1:G1 → H1`) | `oovra compose` | Stage 3 |
| SPLIT (`B3 → C3:H3`) | `oovra decompose` | Stage 3 |
| FORWARD-DIFF (`B10, C10 → D10`) | `oovra compare` | Stage 4 |
| (rewrite all cells) | `oovra migrate` | Stage 5 |

Stage 1 builds the parser and schema that every operator depends on.

The Sheet analog: each *white cell* in your spreadsheet is a `.md` file. Each *yellow output cell* is a new file Oovra writes. The *formula* is the Rust operator. The structural difference: the Sheet operates on string values; Oovra operates on **typed parsed structures**, with the format itself enforcing the types.

---

## Part 0 — The Format Decision: Why TOML

Before any code, the load-bearing decision: what format does the frontmatter use? Three real candidates:

### YAML — the obvious default

YAML is the most common Markdown frontmatter format (Jekyll, Hugo with `---`, Obsidian, mdBook). LLM agents author it with very high first-shot correctness — there's vastly more YAML in their training data than TOML. Obsidian's Properties UI requires YAML.

But YAML has three problems specific to **this** system:

1. **Version-as-float.** `version: 1.0` parses as a `float`, not a string. You'd have to either always quote (`version: "1.0.0"`) or accept that `1.0` and `1.0.0` are different types. Agents will sometimes omit quotes; the silent coercion compounds.
2. **The Norway problem.** `tags: [no, off, yes, on]` becomes booleans in YAML 1.1. YAML 1.2 fixed the explicit forms but the ecosystem is mixed.
3. **Multiple ways to write the same thing.** Block-style and flow-style mappings produce the same data. That's fine until you want round-trip equality testing — then you have to canonicalize, and canonicalization is a thing you write and maintain.

### JSON — universal but hostile

JSON is parseable everywhere. But it's hostile to humans (no comments, mandatory quotes, no trailing commas, painful multi-line strings) and looks bad in Markdown frontmatter. The `composed_of = [{id="x"}]` central data structure becomes `"composed_of": [{"id": "x"}]` — more punctuation, less readability. JSON's only unique win is "every parser speaks it"; Rust's `toml` crate is just as mature as `serde_json`.

### TOML — picked

TOML wins for three reasons specific to this system:

1. **`composed_of = [{ id = "...", version = "..." }, ...]` is what TOML inline tables were invented for.** This is the central data structure of the system, and TOML expresses it more naturally than the YAML or JSON alternatives.
2. **Strict, no-coercion typing.** A quoted string stays a quoted string. An array stays an array. The format does the static typing the data needs, so Decompose can recover exactly what Compose wrote.
3. **`+++` parser disambiguation.** A tool can pick the right frontmatter parser from line 1 without peeking inside the block.

The cost: Obsidian's Properties UI doesn't render TOML frontmatter as fields. Body wikilinks, graph view, and search all still work — but properties show as raw code. This is a real but acceptable tradeoff. Oovra is not Obsidian-first; the Rust tool is the design center.

> ★ **Insight — Format choice as type discipline.** Picking TOML over YAML here is fundamentally a type-system decision masquerading as a syntax decision. YAML's implicit coercions are convenient when you're hand-typing config; they're a liability when you're round-tripping machine-generated data through a parser that has to recover exact types. The format is doing static-typing work for you.

### TOML primer (the parts you actually need)

If you've never written TOML, you need to know about five things:

```toml
# 1. Strings — always quote them
name = "Strict Refusal Policy"

# 2. Integers and booleans — bare
body_level = 1
enabled = true

# 3. Arrays — square brackets
tags = ["safety", "production"]

# 4. Inline tables — curly braces (must be on one line)
input = { id = "role-declaration", version = "1.0.0" }

# 5. Arrays of inline tables — combination
composed_of = [
  { id = "role-declaration", version = "1.0.0" },
  { id = "tone-direct", version = "1.0.0" }
]
```

That's 95% of what you'll write. The full TOML spec at [toml.io/en/v1.0.0](https://toml.io/en/v1.0.0) is short and worth a 20-minute read once.

The TOML crate in Rust round-trips inline tables and arrays-of-tables automatically with `serde`. You write a Rust struct; serde reads/writes the TOML form. That's the entire integration.

---

## Part 1 — Architectural Decisions Locked In

Internalize these before writing code. They're load-bearing.

**One file format only.** Every Oovra artifact is a `.md` file with TOML frontmatter delimited by `+++`. Not JSON. Not YAML. Not a second format. Preserve this ruthlessly.

**One schema, two kinds.** Every Oovra element is one struct discriminated by a `kind` string. `kind = "atom"` is a hand-authored element (no recipe). `kind = "compound"` is a composition produced by `oovra compose` (has a recipe). The collapse buys us:

- One parser, one validator, one serializer. The kind enum is a small dispatch at the leaf of the codebase, not a type-level fork.
- Compositional depth generalizes naturally — a compound's `body_level` and `depth` count *physical delimiter levels* and *recipe-tree height* respectively. Nothing in the schema caps depth at 1; deep compositions fall out for free.
- The dispatching that *does* need to happen (Compare's content-vs-structural diff) reads one enum instead of branching on per-field presence.

The cost: some fields (`composed_of`, `generated_at`, `render_mode`, `body_level`, `depth`) are jointly required-or-forbidden depending on the kind. That constraint moves from the type system into a runtime validator. Acceptable.

**The body is the source of truth, the header is the cheap query path.** When Compose joins inputs, the body of the output contains the **complete file content** (frontmatter + body) of every input, wrapped in chiral delimiters. The header has a `composed_of` field listing immediate-input IDs and versions for fast lookup, but the body is what makes `decompose --full` losslessly recover every leaf. This is the property that lets you hand someone a deeply-nested compound and have them reconstruct every element at every level with no library access.

**Four operators, one binary.** `oovra create | compose | decompose | compare`. Validation is internal — there is no `validate` or `inspect` subcommand because `oovra compose --text <id>` already loads, validates, and prints any single element without writing anything to disk.

> ★ **Insight — Body-tree vs header-tree.** Two competing places to store the recursion: the header (TOML inline tables, deeply nested) or the body (concatenated complete files, naturally self-similar). The body wins because TOML 1.0 inline tables can't span lines — a deeply nested `composed_of = [{...,composed_of=[{...,...}]}]` would be one thousand-character line at depth 5. Moving the recursion into the body sidesteps the constraint entirely. The header always describes one level; the body describes all levels.

---

## Part 2 — Build Stages

### Stage 1 — Parser and Schema

This is the foundation. Every operator depends on it. Get it right.

#### 1.1 — Specify the schema on paper

Before any code, write down the fields. From the working code's `src/header.rs`:

```rust
pub enum PromptElementKind { Atom, Compound }

pub struct PromptElementHeader {
    pub name: String,        // always required
    pub kind: PromptElementKind, // always required ("atom" or "compound")
    pub id: String,          // always required, kebab-case
    pub version: String,     // always required, semver
    pub meta: String,        // always required, may be ""

    // Required when kind = "compound" (and forbidden when kind = "atom"):
    pub generated_at: Option<String>,         // RFC 3339
    pub render_mode: Option<String>,          // e.g. "markdown-h2"
    pub body_level: Option<u32>,              // physical delimiter level (>= 1)
    pub depth: Option<u32>,                   // recipe-tree height (mirrors body_level)
    pub composed_of: Option<Vec<InputRef>>,
}

pub struct InputRef {
    pub id: String,
    pub version: String,
}
```

Five always-required fields, five more for compounds. The required fields (`name`, `kind`, `id`, `version`, `meta`) are the minimum identity of an element. `meta` is required as a field but may be empty — keeping the schema rectangular is cheaper than making it optional and dealing with the missing-vs-empty-string question downstream.

> ★ **Insight — Two integers, not one.** `body_level` and `depth` look identical (they're numerically equal on every valid compound) but answer different questions. `body_level` is the **physical delimiter level** used inside the body — always `max(input.body_level, default = 0) + 1`, and it's what the body parser scans for. `depth` is a **human-friendly compositional-depth label** — `max(input.depth, default = 0) + 1`. They are exposed as separate fields so a future renderer can change the delimiter convention without renaming the metadata users read. For all the cases the v0.2 codebase produces, the two coincide; the duplication is intentional decoupling.

#### 1.2 — Sketch one example of each kind

Hand-write one atom and one compound in a notebook:

```toml
+++
name = "Strict Refusal Policy"
kind = "atom"
id = "refusal-policy-strict"
version = "1.0.0"
meta = "Brief, non-preachy decline of harmful requests."
+++

When asked to produce content that would cause concrete harm,
decline briefly and without lecturing. One clear sentence is enough.
```

Are you happy with how that reads? If not, fix the schema before writing code. After Stage 2 ships, format changes get expensive.

#### 1.3 — The delimiter convention

`+++` to open, `+++` to close. Both on their own lines, nothing else. The opening must be line 1. Exactly one blank line follows the close, then the body.

Why `+++` and not `---`: `---` is the YAML/Jekyll convention. Using `+++` (the Hugo convention) signals "this is TOML frontmatter" unambiguously, so a downstream tool never has to peek inside the block to know which parser to use.

#### 1.4 — Scaffold the Cargo project

```bash
cargo new oovra
cd oovra
```

Edit `Cargo.toml`:

```toml
[workspace]            # Mark this as a workspace root so cargo doesn't walk
                       # up looking for a parent workspace. Skip if your
                       # parent dirs don't have stray Cargo.toml files.

[package]
name = "oovra"
version = "0.2.0"
edition = "2021"
description = "A Rust tool for composition and comparison of agentic system prompts from Markdown+TOML 'prompt elements'"
license = "MIT OR Apache-2.0"

[dependencies]
serde       = { version = "1", features = ["derive"] }
toml        = "0.8"
walkdir     = "2"
clap        = { version = "4", features = ["derive"] }
anyhow      = "1"
thiserror   = "1"
semver      = "1"
similar     = "2"
owo-colors  = "4"
chrono      = { version = "0.4", default-features = false, features = ["clock", "serde"] }
serde_json  = "1"

[lib]
name = "oovra"
path = "src/lib.rs"

[[bin]]
name = "oovra"
path = "src/main.rs"
```

A `[lib]` and `[[bin]]` together let you write integration tests against the library without exporting through the binary. The library is also reusable from other Rust code — useful later if you build an `oovra serve` mode.

Module layout (each gets its own file in `src/`):

| Module | Responsibility |
|---|---|
| `error` | `OovraError` enum with `thiserror` |
| `header` | `PromptElementHeader`, `InputRef`, `PromptElementKind`, `LegacyHeader`, validators |
| `element` | parser, splitter, serializer, body delimiter functions, `ParseOptions` |
| `library` | recursive directory loader (legacy-aware) |
| `render` | Compose pipeline + `compute_body_level` + `compute_depth` |
| `decompose` | one-level + `--full` recursive folder writer + `decompose_with` |
| `diff` | Compare with kind-aware, **sequence-aware** structural dispatch |
| `create` | scaffold + label (both always produce atoms) |
| `migrate` | Stage 5: walk a library, rewrite each file in v0.2 schema in place |
| `lib` | public re-exports |
| `main` | clap-derive CLI |

Keep modules small and single-purpose. If a module starts pushing past 400 lines, split it. As of the v0.2 release, `element.rs` is the largest at ~590 lines (the joint validator is dense); the others are well under 300.

#### 1.5 — Define the data structures

The header struct uses serde derive to handle TOML round-tripping:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptElementKind {
    Atom,
    Compound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRef {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptElementHeader {
    pub name: String,
    pub kind: PromptElementKind,
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub meta: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_mode: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_level: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composed_of: Option<Vec<InputRef>>,
}
```

Two important serde annotations:

- `#[serde(default)]` on `meta` lets the field be omitted from TOML and still deserialize (using `String::default()`, i.e. `""`). Without this, an atom missing `meta` would fail to parse.
- `#[serde(default, skip_serializing_if = "Option::is_none")]` on all five conditional fields means: deserialize as `None` when missing, and *omit entirely* when serializing if `None`. Without `skip_serializing_if`, serde would emit them as TOML `null` (which TOML doesn't have, so it'd error).

> ★ **Insight — Why `Option<T>` with `skip_serializing_if`.** This pattern is how you express "this field exists for some elements and not others" in a single struct without resorting to a tagged enum. The runtime validator (Step 1.6) enforces the joint invariant "`kind = Compound` implies all five companion fields are present; `kind = Atom` requires all five absent" — the type system can't express it directly, but the validator covers it before the struct is handed to anyone.

Then a small companion `PromptElement` type that pairs the header with the body:

```rust
pub struct PromptElement {
    pub header: PromptElementHeader,
    pub body: String,
    pub source_path: Option<PathBuf>,  // for better error messages later
}
```

#### 1.6 — Write the parser

The parser is a four-step pipeline:

1. **Split frontmatter from body.** Scan for `+++` lines.
2. **Parse the frontmatter as TOML.** `toml::from_str` into `PromptElementHeader`.
3. **Validate semantics.** Kebab-case, semver, RFC 3339, kind-required-fields, non-empty body.
4. **Return a `PromptElement`** with everything attached.

The split is purely lexical:

```rust
pub fn split_frontmatter(content: &str, path: &Path) -> Result<(String, String)> {
    let mut lines = content.lines();

    let first = lines.next();
    match first {
        Some(line) if line.trim_end() == "+++" => {}
        Some(line) => {
            return Err(OovraError::MissingOpenDelimiter {
                path: path.to_path_buf(),
                actual: line.to_string(),
            });
        }
        None => return Err(OovraError::EmptyFile(path.to_path_buf())),
    }

    let mut fm_lines: Vec<&str> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut closed = false;

    for line in lines {
        if !closed {
            if line.trim_end() == "+++" {
                closed = true;
            } else {
                fm_lines.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    if !closed {
        return Err(OovraError::MissingCloseDelimiter(path.to_path_buf()));
    }

    // Consume exactly one blank line after the closing delimiter, if present.
    let body_start = if body_lines.first().map(|l| l.trim().is_empty()).unwrap_or(false) {
        1
    } else {
        0
    };

    let body = body_lines[body_start..].join("\n");
    let frontmatter = fm_lines.join("\n");

    Ok((frontmatter, body))
}
```

**Why a custom splitter and not `gray_matter`?** The original spec recommends the `gray_matter` crate for the split-and-parse step. We chose a custom splitter because it gives us precise, agent-readable error messages (`"Missing opening '+++' delimiter on line 1 of nodes/refusal.md. Expected '+++', got 'name = ...'"`). The library's generic "no frontmatter found" doesn't tell an LLM agent what to fix. The 25 lines of custom code are worth it.

> ★ **Insight — Error messages are part of the API.** Because LLM agents will produce Oovra files, the error messages they see are part of the agent-facing API. An error like `"Field 'version' missing in nodes/refusal.md"` is actionable; `"TOML parse failed"` is not. Spend extra effort here — it pays off every time an agent has to self-correct.

The TOML parse step is one line:

```rust
let header: PromptElementHeader = toml::from_str(&fm_str)
    .map_err(|source| OovraError::InvalidToml { path: path.to_path_buf(), source })?;
```

Then validate. Three families of checks, applied in this order:

```rust
fn validate_header(header: &PromptElementHeader, body: &str, path: &Path) -> Result<()> {
    // Lexical: id must be kebab-case
    if !is_kebab_case(&header.id) {
        return Err(OovraError::InvalidField {
            path: path.to_path_buf(),
            field: "id",
            value: header.id.clone(),
            reason: "must be kebab-case (...)".to_string(),
        });
    }

    // Lexical: version must be semver
    if !is_valid_semver(&header.version) {
        return Err(OovraError::InvalidField { /* ... */ });
    }

    // Name must be non-empty after trim
    if header.name.trim().is_empty() {
        return Err(OovraError::InvalidField { /* ... */ });
    }

    // Body must be non-empty
    if body.trim().is_empty() {
        return Err(OovraError::EmptyBody(path.to_path_buf()));
    }

    // Joint invariant: compounds have all four required companion fields
    // (composed_of, generated_at, render_mode, body_level) plus an optional
    // depth; atoms have none of them. Kind drives the dispatch.
    match header.kind {
        PromptElementKind::Atom => validate_atom(header, path),
        PromptElementKind::Compound => validate_compound(header, path),
    }
}

fn validate_atom(header: &PromptElementHeader, path: &Path) -> Result<()> {
    // Per-field check: each compound-only field on an atom is its own error
    // arm, so the message names the exact offending field.
    let forbidden: &[(&'static str, bool)] = &[
        ("composed_of", header.composed_of.is_some()),
        ("generated_at", header.generated_at.is_some()),
        ("render_mode", header.render_mode.is_some()),
        ("body_level", header.body_level.is_some()),
        ("depth", header.depth.is_some()),
    ];
    for (field, present) in forbidden {
        if *present {
            return Err(OovraError::AtomHasForbiddenField {
                path: path.to_path_buf(),
                id: header.id.clone(),
                field,
            });
        }
    }
    Ok(())
}

fn validate_compound(header: &PromptElementHeader, path: &Path) -> Result<()> {
    // composed_of must be present + non-empty; every entry must be a
    // kebab-case id with a semver version.
    let composed_of = header.composed_of.as_ref().ok_or_else(|| {
        OovraError::CompoundMissingField { path: path.to_path_buf(), id: header.id.clone(), field: "composed_of" }
    })?;
    // generated_at must be present and RFC 3339; render_mode and body_level
    // must be present; body_level >= 1; depth (if present) >= 1.
    // ... (see src/element.rs for the full body of these helpers)
    Ok(())
}
```

The two-arm structure encodes a **joint invariant**: a compound has all four required companion fields set (with a real recipe) plus an optional `depth`; an atom has none of them. Anything else is rejected. The dedicated `AtomHasForbiddenField` and `CompoundMissingField` error variants exist specifically so the message names the exact field at fault — "Atom 'foo' has forbidden field 'body_level'" is what an agent needs to self-correct; "validation failed" is not.

> ★ **Insight — Why the joint invariant beats per-field optionality.** Without the `else` branch, an attacker (or a confused agent) could write a file with `kind = "atom"` but a `composed_of` array stuffed full of dangling references — claiming atom-ness while carrying compound machinery. Decompose would either misbehave or refuse, but only when called; in the meantime the file would sit in libraries with two contradictory descriptions of what it is. The rejection-on-mixed-fields rule makes that whole class of confusion impossible at parse time.

The three validators are tiny:

```rust
pub fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('-') || s.ends_with('-') {
        return false;
    }
    if s.contains("--") {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn is_valid_semver(s: &str) -> bool {
    semver::Version::parse(s).is_ok()
}

pub fn is_valid_rfc3339(s: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(s).is_ok()
}
```

Each is delegated to a vetted library (`semver`, `chrono`) where one exists. Don't reinvent these — there are too many edge cases (semver pre-release suffixes, RFC 3339 timezones with fractional seconds).

> ★ **Insight — Algorithm choice: hand-roll kebab-case, delegate semver and RFC 3339.** Three small validation jobs; three different best practices. Kebab-case is a regular language with a five-line state machine — write it yourself; pulling in `regex` would be 200KB of binary for nothing. Semver and RFC 3339 are full grammars with corner cases (semver pre-release tags, RFC 3339 leap seconds, timezone offsets) that have already cost dozens of CVEs in other languages — always use the canonical crate.

#### 1.7 — Write the serializer

The inverse:

```rust
pub fn serialize(element: &PromptElement) -> Result<String> {
    let toml_string =
        toml::to_string_pretty(&element.header).map_err(|source| OovraError::TomlSerialize {
            id: element.header.id.clone(),
            source,
        })?;

    let body_trimmed = element.body.trim_end_matches('\n');
    Ok(format!("+++\n{toml_string}+++\n\n{body_trimmed}\n"))
}
```

Three subtleties:

- `to_string_pretty` (not `to_string`) emits readable TOML with normalized whitespace and array formatting. The output is stable across runs, which matters for round-trip equality testing.
- `trim_end_matches('\n')` then re-append a single `\n` normalizes trailing whitespace. Without this, body strings that end with `\n\n\n\n` produce different output every round-trip.
- The format string uses Rust's named-capture syntax (`{toml_string}`) introduced in 2021 edition.

Always pair `serialize` (pure, returns a String) with a separate `write` function that does the I/O — this keeps the testable part free of `std::fs`.

```rust
pub fn write(element: &PromptElement, path: &Path) -> Result<()> {
    let content = serialize(element)?;
    // Validate first by parsing the in-memory string. If this fails,
    // nothing is touched on disk.
    let _ = parse(&content, path)?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| OovraError::WriteIo {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    fs::write(path, &content).map_err(|source| OovraError::WriteIo {
        path: path.to_path_buf(),
        source,
    })?;
    parse_file(path)?;  // Paranoia: re-read from disk, in case of FS-level corruption
    Ok(())
}
```

**Validate before you write, not after.** Earlier this code only had the post-write `parse_file(path)` check — and it worked, but with a bad UX consequence: a bad input (e.g. an ID that wasn't kebab-case) would write the file *first*, then fail validation, leaving an unparseable orphan on disk. The fix is two-line: serialize, then `parse(&content, path)?` against the in-memory string before any `fs::write` happens. The post-write `parse_file` stays as a paranoia check against filesystem-layer corruption (BOM bytes, line-ending conversion, encoding) — but the user-facing validation never produces an orphan.

> ★ **Insight — Discovered during live testing.** The orphan-file behavior didn't show up in any unit test because tests don't `ls` the filesystem afterwards — they assert on return values. It surfaced the moment a CLI user typed a bad ID and noticed the file was still there. The lesson: tests that only check return values miss whole categories of side-effect bugs. Add at least one test per write-path that asserts on the *resulting filesystem state*, not just the function's return.

#### 1.7.5 — Transitional parse mode for v0.1 files

The v0.2 parser rejects files lacking the `kind` field. But v0.1 files are everywhere — anyone with an existing library has a directory full of `order = N` frontmatters. Two paths exist for handling them:

1. **`oovra migrate`** (Stage 5) — rewrites every file on disk in v0.2 schema. The recommended path for any library you control.
2. **`--legacy` flag** — read-only acceptance of v0.1 files for ad-hoc operations during the transition window. Writes are still always in v0.2 format.

Both rely on the same in-memory infrastructure: a separate `LegacyHeader` struct, a `looks_like_missing_kind` heuristic on the toml parse error, and an `into_v2()` converter.

```rust
#[derive(Debug, Deserialize)]
pub struct LegacyHeader {
    pub name: String,
    pub order: u32,
    pub id: String,
    pub version: String,
    #[serde(default)] pub meta: String,
    #[serde(default)] pub generated_at: Option<String>,
    #[serde(default)] pub render_mode: Option<String>,
    #[serde(default)] pub body_level: Option<u32>,
    #[serde(default)] pub composed_of: Option<Vec<InputRef>>,
}

impl LegacyHeader {
    pub fn into_v2(self) -> Result<PromptElementHeader, String> {
        let kind = match (self.order, self.composed_of.is_some()) {
            (_, true)      => PromptElementKind::Compound,
            (0, false)     => PromptElementKind::Atom,
            (n, false)     => return Err(format!(
                "legacy element '{}' has order = {} but no composed_of; \
                 v0.1 already rejected this case, so it cannot be migrated.", self.id, n)),
        };
        let depth = self.body_level;  // v0.2 invariant: depth == body_level
        Ok(PromptElementHeader {
            name: self.name, kind, id: self.id, version: self.version, meta: self.meta,
            generated_at: self.generated_at, render_mode: self.render_mode,
            body_level: self.body_level, depth, composed_of: self.composed_of,
        })
    }
}
```

The mapping is from SPEC §5.1: `(order=0, no recipe)` → atom; `(any order, has recipe)` → compound; `(order ≥ 1, no recipe)` is impossible — v0.1 rejected it too, so v0.2 refuses to synthesize a kind.

`ParseOptions { legacy: bool }` carries the flag through the parser:

```rust
pub fn parse_with(content: &str, path: &Path, opts: ParseOptions) -> Result<PromptElement> {
    let (fm_str, body) = split_frontmatter(content, path)?;
    let header: PromptElementHeader = match toml::from_str(&fm_str) {
        Ok(h) => h,
        Err(v2_err) => {
            if opts.legacy && looks_like_missing_kind(&v2_err) {
                let legacy: LegacyHeader = toml::from_str(&fm_str)?;
                legacy.into_v2()?
            } else {
                return Err(OovraError::InvalidToml { path: path.to_path_buf(), source: v2_err });
            }
        }
    };
    validate_header(&header, &body, path)?;
    Ok(PromptElement { header, body, source_path: Some(path.to_path_buf()) })
}

fn looks_like_missing_kind(err: &toml::de::Error) -> bool {
    let msg = err.to_string();
    msg.contains("missing field `kind`") || msg.contains("missing field \"kind\"")
}
```

The heuristic on the error message is what avoids needing a thread-local or a separate parser pipeline. It's small and stable: the toml crate's missing-field message is part of its public surface.

> ★ **Insight — A separate struct, not an `Option<u32>` on the main header.** v0.1 and v0.2 schemas differ on what's required, what's forbidden, and (most importantly) what mapping rule turns one into the other. Putting `order` as an optional field on `PromptElementHeader` would mean every operator has to think about "what if both `kind` and `order` are present?" The two-struct design moves that question to a single `into_v2()` call at the parse boundary. Past that boundary, everything is pure v0.2.

#### 1.8 — Write the library loader

```rust
pub struct Library {
    pub root: PathBuf,
    pub elements: HashMap<String, PromptElement>,
}

impl Library {
    pub fn load(root: &Path) -> Result<Self> {
        let mut elements: HashMap<String, PromptElement> = HashMap::new();
        let mut id_to_path: HashMap<String, PathBuf> = HashMap::new();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !entry.file_type().is_file() { continue; }
            if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }

            let element = parse_file(path)?;
            let id = element.header.id.clone();

            if let Some(existing) = id_to_path.get(&id) {
                return Err(OovraError::DuplicateId {
                    id: id.clone(),
                    first: existing.clone(),
                    second: path.to_path_buf(),
                });
            }
            id_to_path.insert(id.clone(), path.to_path_buf());
            elements.insert(id, element);
        }

        Ok(Library { root: root.to_path_buf(), elements })
    }
}
```

**Fail-loud on duplicates.** A library with two elements claiming the same ID has no defined behavior. Refusing to load is the right discipline; it surfaces the problem at library-load time rather than at compose time, where you'd get a confusing "wrong version" error instead.

> ★ **Insight — Why `walkdir` instead of `std::fs::read_dir`.** `read_dir` is a single-level iterator; you'd write your own recursion. `walkdir` handles symlinks, permissions errors (gracefully — you can `.filter_map(Result::ok)` to skip unreadable directories), and platform-specific weirdness (Windows reparse points). It's 200 lines of crate for a function you would otherwise spend a day getting right.

#### 1.9 — Author five real elements

Stop coding. Write five real prompt elements: a role declaration, a refusal policy, an output formatter, a tone instruction, an examples block. Save them in `elements/` and parse them through your loader. You'll discover schema problems only by authoring real content. Fix the schema before continuing.

The codebase ships five canonical examples in `elements/`:

- `role-declaration.md`
- `refusal-policy-strict.md`
- `output-format-markdown.md`
- `tone-direct.md`
- `examples-block.md`

These are your test corpus and your dogfood material.

---

### Stage 2 — Create

The simplest operator. Building it first proves the parse-validate-serialize loop end-to-end before you tackle JOIN and DIFF.

#### 2.1 — Two modes: scaffold and label

```rust
pub fn scaffold(args: ScaffoldArgs) -> Result<PathBuf> {
    let header = PromptElementHeader {
        name: args.name.unwrap_or_else(|| args.id.clone()),
        kind: PromptElementKind::Atom,
        id: args.id.clone(),
        version: args.version,
        meta: args.meta,
        generated_at: None,
        render_mode: None,
        body_level: None,
        depth: None,
        composed_of: None,
    };
    let body = format!(
        "<!-- TODO: write the prompt body for `{}` here. \
         This element is an atom — internally consistent, portable. -->",
        args.id
    );
    let element = PromptElement::new(header, body);
    let path = args.library_dir.join(format!("{}.md", args.id));
    write(&element, &path)?;  // validates in memory FIRST, then writes
    Ok(path)
}
```

Both modes always produce atoms (`kind = "atom"`, no recipe). There is no way to scaffold a compound by hand — that's `compose`'s job. This is a deliberate constraint: the validator (Step 1.6) actively rejects any hand-authored file that claims `kind = "compound"` without a recipe (or `kind = "atom"` with companion fields populated). Note the five `Option<T>` companion fields on the header (`generated_at`, `render_mode`, `body_level`, `depth`, `composed_of`) are *all* `None` here — the validator rejects any subset being `Some` for an atom, keeping the schema rectangular.

The original spec said Create should "delete the file on failure to parse." The shipped implementation goes one step better: it validates the would-be file content *in memory* before any `fs::write` happens, so a failure leaves nothing on disk to clean up. The original "delete on failure" approach has a TOCTOU window between write and delete; the in-memory pre-check has none.

The label mode is similar but takes an existing `.md` file, prepends a header, and writes back. It refuses to overwrite a file that already has an Oovra header unless `--force` is passed:

```rust
if looks_like_oovra_file(&original) && !args.force {
    return Err(OovraError::AlreadyLabeled(args.source_path.clone()));
}
```

> ★ **Insight — `looks_like_oovra_file` is a heuristic, not a parse.** It just checks `content.starts_with("+++\n")`. We don't need to fully parse the existing file at this point — we just need to know whether `--force` is required. Cheap detection beats deep parsing when the only consumer is the user-confirmation path.

#### 2.2 — Wire up the CLI for Stage 2

Use clap's derive feature. The CLI struct mirrors your library functions:

```rust
#[derive(Parser, Debug)]
#[command(name = "oovra", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Create(CreateArgs),
    Compose(ComposeArgs),
    Decompose(DecomposeArgs),
    Compare(CompareArgs),
}
```

Each subcommand's arg struct uses `#[derive(Args)]`. The handler function is thin: parse args, call one library function, print the result. If a handler grows past 30 lines, push logic into the library.

Test end-to-end: `oovra create --new test-element --library ./elements`. Inspect the file. Re-parse it manually with `parse_file`. This is your first proof the system works.

---

### Stage 3 — Compose and Decompose

This is the JOIN/SPLIT pair from your sheet, lifted to typed structures.

#### 3.1 — Compose's input and output

`Compose` takes a list of input IDs (with optional version pins) and a library reference. It produces one compound element. Three output modes:

| Mode | Flag | Output |
|---|---|---|
| File (default) | (none) | A complete Oovra file written to disk |
| Text | `--text` | Clean H2-formatted prompt printed to stdout (NOT a valid Oovra file) |
| Re-render | `--re-render <path>` | Read existing compound file's recipe, re-resolve against current library, overwrite body |

#### 3.2 — The size calculators

Compose computes two numbers from its inputs. Both use the same strict-escalation rule: take the max of the corresponding input field (treating absent fields as 0 — atoms have no `body_level` or `depth` on disk) and add one.

```rust
pub fn compute_body_level(input_body_levels: &[u32]) -> u32 {
    input_body_levels.iter().copied().max().map(|m| m + 1).unwrap_or(1)
}

pub fn compute_depth(input_depths: &[u32]) -> u32 {
    input_depths.iter().copied().max().map(|m| m + 1).unwrap_or(1)
}
```

Both are well-defined on every input shape:

- `compute_body_level(&[0, 0, 0])` = 1 — three atoms, output is body_level 1
- `compute_body_level(&[1, 1])` = 2 — two body_level-1 compounds, output is body_level 2
- `compute_body_level(&[1, 0, 0, 0])` = 2 — one body_level-1 compound plus three atoms; output strictly escalates
- `compute_body_level(&[2, 1, 0, 0, 0, 0, 0])` = 3 — mixed inputs, strict escalation off the max

The strict-escalation rule is what keeps the on-disk delimiter scheme unambiguous: the outer parser scans for exactly `body_level + 1` tildes, and inner chunks (with strictly fewer tildes) are inert text. See [kind-and-delimiters.md](./kind-and-delimiters.md).

> ★ **Insight — `depth` and `body_level` are the same number on disk, exposed under two names.** This is intentional decoupling for future flexibility: a future renderer might count physical delimiter levels differently from recipe-tree height (e.g. by combining chunks within a single delimiter envelope). Exposing both lets downstream tooling read whichever number it cares about without leaking the convention.

#### 3.3 — The resolver

For each input ID, look it up in the library and verify any version pin:

```rust
for (id, pin) in &req.inputs {
    let element = req.library.get(id)
        .ok_or_else(|| OovraError::ElementNotFound { id: id.clone() })?;

    if let Some(pin) = pin {
        if &element.header.version != pin {
            return Err(OovraError::VersionMismatch {
                id: id.clone(), pin: pin.clone(),
                actual: element.header.version.clone(),
            });
        }
    }
    resolved.push(element);
    input_refs.push(InputRef::new(id.clone(), element.header.version.clone()));
}
```

**Pin semantics in v0.2: exact-string match.** Don't implement semver range matching (`^1.2.0`, `>=1.0.0, <2.0.0`) until you actually need it. The `semver` crate is in your dependencies for when you do; YAGNI until then.

#### 3.4 — The body renderer (the load-bearing part)

The body of a compound element is the **concatenation of full-file content of each input, wrapped in chiral level-aware delimiters**. This is what makes the file losslessly self-describing.

Delimiter functions:

```rust
pub fn body_delimiter_open(body_level: u32) -> String {
    format!("{}>>", "~".repeat((body_level + 1) as usize))
}

pub fn body_delimiter_close(body_level: u32) -> String {
    format!("{}<<", "~".repeat((body_level + 1) as usize))
}
```

Level-1 delimiters: `~~>>` / `~~<<`. Level 2: `~~~>>` / `~~~<<`. Level N: `(N+1)` tildes + chiral suffix. The suffix is chiral so open and close can never be confused; the tilde count is strictly monotonic so an outer parser scanning for level-N ignores inner level-(N−k).

The `body_level` for a Compose output is `max(input.body_level, default = 0) + 1`. Atoms contribute `0` (they have no `body_level` on disk). The strict-escalation rule means the outer delimiter always has strictly more tildes than any inner element's body delimiter — that is what makes recursive decomposition unambiguous.

(The `compute_body_level` helper itself was defined in [Section 3.2](#32--the-size-calculators); the renderer simply consumes its result.)

The renderer takes `body_level` directly:

```rust
fn wrap_chunk(body_level: u32, full_file_content: &str) -> String {
    let open = body_delimiter_open(body_level);
    let close = body_delimiter_close(body_level);
    let trimmed = full_file_content.trim_end_matches('\n');
    format!("{open}\n{trimmed}\n{close}")
}

pub fn render_body(body_level: u32, input_files: &[String]) -> String {
    input_files.iter()
        .map(|f| wrap_chunk(body_level, f))
        .collect::<Vec<_>>()
        .join("\n")
}
```

For each input, **`serialize(input)` to get its complete file string** (frontmatter + body), wrap it in level-`body_level` delimiters, join all chunks with newlines.

> ★ **Insight — The escalation rule does the real work, and `body_level` is what enforces it.** The outer file's delimiter always has strictly more tildes than any embedded element's body delimiter. The outer parser scans for `(body_level + 1)` tildes — inner delimiters with fewer tildes are inert text. This is what makes the recursive splitting unambiguous regardless of depth. **The bug history**: an earlier (v0.1) version conflated two distinct quantities — a count-based "logical order" and the physical delimiter level — into a single `order` field. When the count-based formula produced a value that did not strictly exceed `max(input.order)`, the outer delimiter collided with inner delimiters. v0.2 dropped the count-based formula entirely; `body_level` strictly escalates by construction, and the field doubles as the "compositional depth" label.

#### 3.5 — Stitching it all together

The full Compose function:

```rust
pub fn compose(req: ComposeRequest<'_>) -> Result<PromptElement> {
    if req.inputs.is_empty() { return Err(OovraError::EmptyCompose); }

    // 1. Resolve each input against the library
    let mut resolved: Vec<&PromptElement> = Vec::with_capacity(req.inputs.len());
    let mut input_refs: Vec<InputRef> = Vec::with_capacity(req.inputs.len());
    for (id, pin) in &req.inputs {
        let element = req.library.get(id).ok_or_else(/* ... */)?;
        if let Some(pin) = pin { /* check pin */ }
        resolved.push(element);
        input_refs.push(InputRef::new(id.clone(), element.header.version.clone()));
    }

    // 2. Compute body_level (physical delimiter level) and depth (recipe tree height).
    //    Both follow the strict-escalation rule. Atoms contribute 0 in both calculations.
    let input_body_levels: Vec<u32> =
        resolved.iter().map(|e| e.header.body_level.unwrap_or(0)).collect();
    let input_depths: Vec<u32> =
        resolved.iter().map(|e| e.header.depth.unwrap_or(0)).collect();
    let body_level = compute_body_level(&input_body_levels);
    let depth = compute_depth(&input_depths);

    // 3. Render the body using body_level
    let mut input_files: Vec<String> = Vec::with_capacity(resolved.len());
    for input in &resolved {
        input_files.push(serialize(input)?);
    }
    let body = render_body(body_level, &input_files);

    // 4. Construct the header
    let header = PromptElementHeader {
        name: req.output_name,
        kind: PromptElementKind::Compound,
        id: req.output_id,
        version: req.output_version,
        meta: req.output_meta,
        generated_at: Some(Utc::now().to_rfc3339()),
        render_mode: Some("markdown-h2".to_string()),
        body_level: Some(body_level),
        depth: Some(depth),
        composed_of: Some(input_refs),
    };

    Ok(PromptElement::new(header, body))
}
```

Four-step pipeline: resolve, compute-both-integers, render-with-body-level, build-header. The strict-escalation rule on `body_level` ensures the on-disk delimiter level always escalates, eliminating the mixed-input delimiter collision that an earlier (v0.1) count-based formula was vulnerable to.

#### 3.6 — Decompose

Decompose is the inverse, and it's almost trivial because the body is already structured. Read the file, split the body at level-`body_level` delimiters (taken from the header), parse each chunk as a complete Oovra file:

```rust
pub fn decompose(element: &PromptElement) -> Result<Vec<PromptElement>> {
    if element.header.is_atom() {
        return Err(OovraError::CannotDecomposeAtom {
            id: element.header.id.clone(),
        });
    }

    let body_level = element.header.body_level.ok_or_else(|| {
        OovraError::CompoundMissingField {
            id: element.header.id.clone(),
            field: "body_level",
        }
    })?;

    let open = body_delimiter_open(body_level);
    let close = body_delimiter_close(body_level);

    let mut chunks: Vec<String> = Vec::new();
    let mut current: Option<Vec<&str>> = None;

    for line in element.body.lines() {
        let trimmed = line.trim_end();
        if trimmed == open {
            if current.is_some() {
                return Err(OovraError::BodyParse { /* unmatched-open arm */ });
            }
            current = Some(Vec::new());
        } else if trimmed == close {
            match current.take() {
                Some(buf) => chunks.push(buf.join("\n")),
                None => return Err(OovraError::BodyParse { /* unmatched-close arm */ }),
            }
        } else if let Some(buf) = current.as_mut() {
            buf.push(line);
        }
    }

    // ... plus two more BodyParse arms: missing final close, no opens found at all.

    // Parse each chunk as a complete Oovra file
    let mut parsed: Vec<PromptElement> = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        parsed.push(parse(chunk, /* synthetic path */)?);
    }
    Ok(parsed)
}
```

The state machine: walk the body line by line. When you see an open delimiter, start collecting. When you see a close, finalize the chunk. Otherwise, append to the current chunk if you're inside one. The `BodyParse` arms (sketched as `/* ... arm */` above) cover the four ways a body can be malformed: open-while-open, close-without-open, missing-final-close, and zero-opens — see `src/decompose.rs` for the full text of each error message.

`decompose --full` is recursive:

```rust
pub fn decompose_full(element: &PromptElement, out_root: &Path) -> Result<PathBuf> {
    let element_dir = out_root.join(&element.header.id);
    fs::create_dir_all(&element_dir)?;
    write_recursive(element, &element_dir)?;
    Ok(element_dir)
}

fn write_recursive(element: &PromptElement, dir: &Path) -> Result<()> {
    // Write this element to dir/<id>.md
    let element_path = dir.join(format!("{}.md", element.header.id));
    fs::write(&element_path, serialize(element)?)?;

    if element.header.is_atom() { return Ok(()); }

    // For each immediate input, write it directly (atom leaf) or recurse (compound sub-tree)
    for input in decompose(element)? {
        if input.header.is_atom() {
            fs::write(dir.join(format!("{}.md", input.header.id)), serialize(&input)?)?;
        } else {
            let sub_dir = dir.join(&input.header.id);
            fs::create_dir_all(&sub_dir)?;
            write_recursive(&input, &sub_dir)?;
        }
    }
    Ok(())
}
```

For a `body_level = 2` compound with two `body_level = 1` inputs, you get:

```
out/<root-id>/
├── <root-id>.md
├── <input1-id>/
│   ├── <input1-id>.md
│   ├── <leaf-a>.md
│   └── <leaf-b>.md
└── <input2-id>/
    ├── <input2-id>.md
    ├── <leaf-c>.md
    └── <leaf-d>.md
```

Atoms are flat `.md` files (nothing to recurse into). Compounds get their own subdirectory.

> ★ **Insight — The recursion is a self-similar fixed-point.** The body parser splits at level-N delimiters and re-runs the *same* file parser on each chunk. There's no special-casing per kind, no recursion depth limit (other than Rust's stack), no separate "deep parser." `parse` is `parse` whether the chunk is an atom or a deeply-nested compound.

#### 3.7 — The `--text` renderer (a different problem)

`--text` is for "give me a prompt to paste into a model" — it should NOT use the Oovra delimiters. It produces clean Markdown with H2 headers per leaf:

```rust
pub fn render_text(inputs: &[&PromptElement]) -> Result<String> {
    let parts: Vec<String> = inputs.iter()
        .map(|e| render_for_paste(e))
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("\n\n"))
}

fn render_for_paste(element: &PromptElement) -> Result<String> {
    if element.header.is_atom() {
        return Ok(format!("## {}\n\n{}", element.header.id, element.body.trim()));
    }
    // Compound: descend recursively, flattening to a list of atom leaves
    let subs = crate::decompose::decompose(element)?;
    let parts: Vec<String> = subs.iter().map(render_for_paste).collect::<Result<Vec<_>>>()?;
    Ok(parts.join("\n\n"))
}
```

Two renderers, two purposes:
- `render_body` produces the on-disk Oovra-format body (with delimiters, for round-tripping).
- `render_text` produces a human-pasteable prompt (no delimiters, just H2-formatted prose).

The original spec called this "render mode." `render_body` is hardcoded to the Oovra-delimited form (the only on-disk form supported), and `render_text` is hardcoded to the H2 form. A future version might add a `--render-mode=claude-xml` to get `<role>...</role>` wrapping. The two-renderer design makes that an additive change, not a refactor.

---

### Stage 4 — Compare

The FORWARD-DIFF operator with **kind-aware dispatch**: an atom can only be compared against an atom, and a compound against a compound.

#### 4.1 — Three comparison cases

Compare's first job is dispatching:

```rust
pub fn compare(a: &PromptElement, b: &PromptElement) -> Result<DiffReport> {
    match (a.header.is_atom(), b.header.is_atom()) {
        (true, true)   => Ok(DiffReport::Content(content_diff(a, b))),
        (false, false) => Ok(DiffReport::Structural(structural_diff(a, b)?)),
        (a_atom, _) => Err(OovraError::KindMismatch {
            a_id: a.header.id.clone(),
            a_kind: if a_atom { "atom" } else { "compound" },
            b_id: b.header.id.clone(),
            b_kind: if a_atom { "compound" } else { "atom" },
        }),
    }
}
```

Three paths:
- **Both atoms** → content diff (body unified diff + metadata changes).
- **Both compounds** → structural diff over `composed_of` (added / removed / version-changed).
- **Mixed kinds** → `KindMismatch` error.

> ★ **Insight — Why `kind` is the single dispatch axis (v0.2 simplification).** v0.1 carried a separate `order` integer and had to check both "orders match" and "kinds match" before routing to the right diff. In practice, two atoms compared at "different orders" never happens (atoms are always order 0), and a kind-mismatch already explains every interesting mismatch case. v0.2 collapses dispatch to a single `kind` match. The earlier two-axis check is gone, and the codebase is the smaller for it.

#### 4.2 — Content diff (atoms)

Use the `similar` crate for the body unified diff:

```rust
fn content_diff(a: &PromptElement, b: &PromptElement) -> ContentDiff {
    let mut field_changes = Vec::new();
    track_field(&mut field_changes, "name", &a.header.name, &b.header.name);
    track_field(&mut field_changes, "version", &a.header.version, &b.header.version);
    track_field(&mut field_changes, "meta", &a.header.meta, &b.header.meta);

    let diff = similar::TextDiff::from_lines(&a.body, &b.body);
    let body_unified_diff = diff.unified_diff()
        .header(&a.header.id, &b.header.id)
        .to_string();

    ContentDiff { /* ... */ }
}
```

`similar::TextDiff::from_lines` produces line-by-line LCS-based diffs — the same algorithm Git uses. The `unified_diff()` builder formats it in the standard `+/-/@@` form you can colorize for terminal output.

> ★ **Insight — Why LCS-based diff (`similar`) instead of word-level or character-level.** Prompt bodies are paragraph-shaped. Line-level diff matches the unit a human author edits in. Word-level diff over multi-line bodies produces visually noisy reports where a small edit looks like a wholesale rewrite. Match the diff granularity to the editing granularity.

#### 4.3 — Structural diff (compounds) — sequence-aware

This is where Compare earns its keep. The structural diff operates on the `composed_of` arrays. In v0.2 the diff is **sequence-aware**: position changes are surfaced as a fourth axis (`moved`) alongside `added`, `removed`, and `version_changed`.

```rust
fn structural_diff(a: &PromptElement, b: &PromptElement) -> Result<StructuralDiff> {
    let a_inputs = a.header.composed_of.as_ref().expect("compound has composed_of");
    let b_inputs = b.header.composed_of.as_ref().expect("compound has composed_of");

    // Index each side by id → (position, version). The HashMap gives O(1)
    // lookup; carrying position alongside version is what makes the diff
    // sequence-aware.
    let a_by_id: HashMap<&str, (usize, &str)> = a_inputs.iter().enumerate()
        .map(|(pos, i)| (i.id.as_str(), (pos, i.version.as_str()))).collect();
    let b_by_id: HashMap<&str, (usize, &str)> = b_inputs.iter().enumerate()
        .map(|(pos, i)| (i.id.as_str(), (pos, i.version.as_str()))).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut version_changed = Vec::new();
    let mut moved = Vec::new();

    // Walk b: classify each b-input as added (id not in a), version_changed
    // (id in a, version differs), and/or moved (id in a, position differs).
    // version_changed and moved are NOT mutually exclusive — a single input
    // can both have its pin bumped and shift position; both axes report it.
    for (b_pos, input) in b_inputs.iter().enumerate() {
        match a_by_id.get(input.id.as_str()) {
            Some(&(a_pos, a_ver)) => {
                if a_ver != input.version.as_str() {
                    version_changed.push(VersionChange { /* ... */ });
                }
                if a_pos != b_pos {
                    moved.push(Move {
                        id: input.id.clone(),
                        version: input.version.clone(),
                        before_pos: a_pos,
                        after_pos: b_pos,
                    });
                }
            }
            None => added.push(PositionedInput { position: b_pos, input: input.clone() }),
        }
    }
    // Walk a: classify each a-input absent from b as removed.
    for (a_pos, input) in a_inputs.iter().enumerate() {
        if !b_by_id.contains_key(input.id.as_str()) {
            removed.push(PositionedInput { position: a_pos, input: input.clone() });
        }
    }

    let recipes_equal = added.is_empty() && removed.is_empty()
        && version_changed.is_empty() && moved.is_empty();

    Ok(StructuralDiff { added, removed, version_changed, moved, recipes_equal, /* ... */ })
}
```

Hash-by-id lookup in O(1) per input. Four categories:
- **Added** — id present in B, absent in A. Position is B-side.
- **Removed** — id present in A, absent in B. Position is A-side.
- **Version-changed** — same id in both, different version.
- **Moved** — same id and version, different position.

`recipes_equal` is true iff all four lists are empty.

This is the diff that makes "two compositions whose rendered bodies are wildly different" reveal themselves as "actually identical except for one input version bump and a reorder." That's the core architectural payoff: **structural similarity through surface noise**.

> ★ **Insight — Why v0.2 made the diff sequence-aware.** v0.1 treated `composed_of` as a set keyed by ID — reordering inputs wasn't reported as a diff. But composing `[role, safety, tone]` then `[tone, safety, role]` produces two files whose *rendered prompts* differ (the model sees different ordering), so the recipes are not semantically equal. The set-based answer was misleading. v0.2 carries position alongside version in the hash, surfaces position differences as a separate `moved` axis, and explicitly documents that `version_changed` and `moved` can fire on the *same input* — there is no false dichotomy between "changed version" and "changed position." See [demos/v0.2/01-operator-gamut](../demos/v0.2/01-operator-gamut/) for the four-axes-all-firing scenario verified end-to-end.

> ★ **Insight — Duplicate-id limitation.** The HashMap key is the id; if a single `composed_of` contains the same id twice, the second occurrence wins in the map, and moves involving duplicate-id inputs are not detected. Duplicate ids are rare (and arguably a smell) — a full LCS-based diff that handles duplicates is on the v0.3 roadmap. Documented at the top of `src/diff.rs`.

#### 4.4 — Wire Compare to the CLI

Print colorized output for humans, JSON for agents:

```rust
fn run_compare(args: CompareArgs) -> anyhow::Result<()> {
    let a = parse_file(&args.a)?;
    let b = parse_file(&args.b)?;
    let report = compare(&a, &b)?;

    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    match report {
        DiffReport::Content(c) => { /* unified diff with red/green colors */ }
        DiffReport::Structural(s) => {
            for pi in &s.added {
                println!("    + [{}] {} @ {}", pi.position, pi.input.id, pi.input.version);
            }
            for pi in &s.removed {
                println!("    - [{}] {} @ {}", pi.position, pi.input.id, pi.input.version);
            }
            for v in &s.version_changed {
                println!("    ~ {} : {} -> {}", v.id, v.before_version, v.after_version);
            }
            for m in &s.moved {
                println!("    ↔ {} @ {} : pos {} -> pos {}",
                         m.id, m.version, m.before_pos, m.after_pos);
            }
        }
    }
    Ok(())
}
```

Use `owo-colors` for the green/red/yellow/blue coding (added is green, removed is red, version_changed is yellow, moved is blue with the `↔` glyph). `serde_json::to_string_pretty(&report)` "just works" because all the diff structs derive `Serialize`.

#### 4.5 — Test every diff axis end-to-end

The integration suite (`tests/end_to_end.rs`) exercises each diff axis in isolation, then together. Three tests are load-bearing:

- **`compare_structural_diff_detects_version_change`** — compose `v1`, bump a library version, compose `v2` with the same ID list, expect exactly one entry in `version_changed`. No add/remove/move.
- **`compare_detects_reorder`** — compose `v1 = [a, b, c]`, compose `v2 = [c, a, b]` against the unchanged library, expect three entries in `moved` and `recipes_equal = false`. (v0.1 would have reported `recipes_equal = true` here — this is the regression test for the sequence-awareness fix.)
- **`compare_distinguishes_move_from_add_remove`** — compose `v1 = [a, b, c]`, compose `v2 = [a, b, d]` where `d` is a different atom, expect `c` on `removed`, `d` on `added`, and `a, b` not on `moved`. This proves `moved` doesn't trigger spuriously when the recipes don't actually overlap at those positions.

A fourth test verifies the non-exclusivity case:

- **`compare_reports_pure_version_change_not_as_move`** — bumping one input's version with no position change should fire `version_changed` only, not `moved`.

If these four pass, your structural diff is doing what it claims. The two composed bodies will have very different timestamps and rendered text, but `compare` cuts through to the structural changes.

---

### Stage 5 — Migrate

The fifth operator is for the schema transition itself. `oovra migrate <library-dir>` walks a directory recursively, parses each `.md` file in legacy mode (accepting both v0.1 and v0.2 shapes), and rewrites every Oovra file in v0.2 schema in place. Idempotent on already-v0.2 files — re-serialization may canonicalize field order but doesn't change content. Non-Oovra `.md` files (no `+++` opener) are skipped.

#### 5.1 — The shape of migration

The function returns a summary so the CLI can print per-file outcomes:

```rust
#[derive(Debug, Default)]
pub struct MigrationSummary {
    pub migrated: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, &'static str)>,   // reason tag
    pub failed:  Vec<(PathBuf, OovraError)>,
}

pub fn migrate_library(root: &Path) -> Result<MigrationSummary> {
    // ... walk root, parse each .md in legacy mode, rewrite in v0.2 ...
}
```

The CLI surface is one positional arg:

```rust
#[derive(clap::Args, Debug)]
struct MigrateArgs {
    /// Library directory to migrate in place. Recursive. Run in a clean
    /// Git working directory so the diff is auditable.
    library: PathBuf,
}
```

#### 5.2 — Recursion into compound bodies (the load-bearing bit)

The naive migrate is one line per file: read it in legacy mode, write it out in v0.2 mode. That works for atoms. But compounds carry their full sub-tree as wrapped sub-files inside the body, and those embedded sub-files have their *own* v0.1 frontmatters. A pure outer-frontmatter rewrite leaves the embedded sub-elements still in v0.1 form — they will then fail v0.2-strict parsing when `decompose` runs on the migrated file.

The fix: recursively decompose the body in legacy mode, migrate each sub-element, re-render the body from the migrated children, and only *then* re-serialize the outer file.

```rust
fn rewrite_with_body_migration(element: &PromptElement) -> Result<PromptElement> {
    if element.header.is_atom() { return Ok(element.clone()); }
    let body_level = element.header.body_level
        .expect("compound validation guarantees body_level");
    let sub_elements = decompose_with(element, ParseOptions { legacy: true })?;
    let mut migrated_chunks: Vec<String> = Vec::with_capacity(sub_elements.len());
    for sub in sub_elements {
        let migrated_sub = rewrite_with_body_migration(&sub)?;
        migrated_chunks.push(serialize(&migrated_sub)?);
    }
    let new_body = render_body(body_level, &migrated_chunks);
    Ok(PromptElement::new(element.header.clone(), new_body))
}
```

This requires a `decompose_with(element, opts)` variant that threads `ParseOptions` through to embedded sub-element parsing — added to `src/decompose.rs` alongside the existing `decompose` (which is now a thin wrapper passing default options). The outer header's `body_level` is preserved from v0.1 (the integer didn't change semantically), so the rendered body lines up exactly.

> ★ **Insight — A bug found by integration testing, not unit testing.** The migrate function's first pass only rewrote outer frontmatter. Unit tests on atoms and shallow compounds passed. The bug surfaced the moment a deep compound was migrated and then `decompose` was run on the result — `decompose` choked on an embedded sub-element with no `kind` field. Lesson: when a migration tool walks a recursive data structure, the test fixtures need to *include* the recursive case, not just the flat case. The regression test `migrate_recursively_rewrites_embedded_frontmatter` now covers this directly.

#### 5.3 — Idempotency and atomicity

Two properties worth verifying explicitly:

- **Idempotent on v0.2 files.** Running `migrate` against an already-migrated directory should be a no-op (modulo possible canonicalization of TOML field order). The integration test `migrate_is_idempotent_on_v0_2_files` confirms this.
- **Atomic per file via in-memory validation.** The same `write` function used by Create and Compose validates the serialized form *in memory* before any disk write — so a file whose migration would produce an invalid v0.2 result is left untouched on disk and reported as `failed`. No partial writes, no torn files.

#### 5.4 — Why `--legacy` exists alongside `migrate`

`--legacy` is the *read-only* counterpart to `migrate`: it lets users run `decompose`, `compare`, or `compose --text` against v0.1 files without committing to a one-way rewrite. Writes are always in v0.2 format regardless of the flag, so a user with mixed libraries can read v0.1 files while gradually migrating them.

The expected lifecycle:

1. v0.2 ships; users discover their old files no longer parse.
2. They run `oovra migrate ./elements` in a clean git working tree, audit the diff, commit.
3. The `--legacy` flag remains available for one minor version (v0.2.x) for stragglers.
4. v0.3 removes `--legacy` and `LegacyHeader`. Users who haven't migrated by then run `oovra migrate` first.

---

## Part 3 — Stop and Use It

You now have a working build. Take a system prompt you actually use — for a coding agent, an Obsidian vault helper, whatever — and break it into elements. Use `oovra create` to scaffold them. Use `oovra compose` to assemble them. Use `oovra compare` to diff against your old hand-written version.

**Use the tool for two weeks before adding any features.** The features you imagined needing will turn out to be wrong; the features you actually need will reveal themselves through use.

---

## Appendix A — Project Layout (the Working Code)

```
oovra/
├── Cargo.toml
├── CHANGELOG.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
├── SCHEMA.md
├── Documentation/
│   ├── README.md
│   ├── reference/                         # feature-by-feature reference docs
│   │   ├── README.md
│   │   ├── schema.md
│   │   ├── kind-and-delimiters.md
│   │   ├── command-create.md
│   │   ├── command-compose.md
│   │   ├── command-decompose.md
│   │   ├── command-compare.md
│   │   ├── errors.md
│   │   └── build-guide.md                 # this file
│   ├── demos/                             # end-to-end demonstrations, split by version
│   │   ├── README.md
│   │   ├── v0.1/                          # 5 demos captured against v0.1, migrated in place
│   │   └── v0.2/                          # v0.2-era demos (operator gamut, sequence-aware compare)
│   └── version-reports/
│       └── v0.1/                          # v0.1 planning artifacts (SPEC-v0.2.md, efficacy, ceiling, scoping)
├── src/
│   ├── lib.rs                             # public re-exports + clippy allow
│   ├── main.rs                            # CLI entry point (clap-derive)
│   ├── error.rs                           # OovraError enum
│   ├── header.rs                          # PromptElementHeader, PromptElementKind, LegacyHeader, InputRef, validators
│   ├── element.rs                         # parser, splitter, serializer, body delimiters, ParseOptions
│   ├── library.rs                         # Library loader (legacy-aware)
│   ├── render.rs                          # Compose, compute_body_level, compute_depth, render_body, render_text
│   ├── decompose.rs                       # decompose, decompose_with, decompose_full, report
│   ├── diff.rs                            # Compare with kind-aware, sequence-aware dispatch
│   ├── create.rs                          # scaffold, label (both always produce atoms)
│   └── migrate.rs                         # migrate_library + recursive frontmatter rewrite
├── elements/                              # 5 sample atoms
│   ├── role-declaration.md
│   ├── refusal-policy-strict.md
│   ├── output-format-markdown.md
│   ├── tone-direct.md
│   └── examples-block.md
└── tests/
    └── end_to_end.rs                      # 19 integration tests covering the full pipeline
```

Total: roughly 2,600 lines of Rust + 860 lines of integration test + 5 sample atoms + the full Documentation tree. The codebase still fits in one head. (Exact counts as of this snapshot: 2,578 src lines, 862 test lines, 11 src modules.)

## Appendix B — Test Strategy

**Unit tests** (`#[cfg(test)] mod tests { ... }` inside each source file): test pure functions. `compute_body_level` and `compute_depth` against tabulated input cases. `is_kebab_case`, `is_valid_semver`, `is_valid_rfc3339` against valid and invalid inputs. `body_delimiter_open` against expected outputs at multiple levels. `parse` round-tripping a minimal file. The atom-validator and compound-validator each get a battery of "what if every individual forbidden field is set" / "what if every individual required field is missing" tests. Legacy-loader tests verify the v0.1→v0.2 mapping for both atoms and compounds (and the rejected `order ≥ 1 without composed_of` case). These tests run in milliseconds and you should run them on every save. v0.2 ships 27 unit tests; if any fails, fix it before touching the integration suite.

**Integration tests** (`tests/end_to_end.rs`): test the full pipeline against real on-disk fixtures. Compose 3 atoms into a `body_level = 1` compound; assert the body contains all expected input IDs. Compose 2 `body_level = 1` compounds into a `body_level = 2` compound; assert both `~~>>` (level 1) AND `~~~>>` (level 2) appear in the body — the strict-escalation rule under test. `decompose_full` a `body_level = 2` element and assert the folder tree has the exact expected structure with full metadata-preservation on every leaf.

Three groups of integration tests carry particular regression weight:

- **`mixed_kind_compose_does_not_collide_with_inner_delimiters`** — exercises a compound whose inputs include both a compound and several atoms; the case that exposed the v0.1 body-level conflation bug.
- **Sequence-aware compare**: `compare_detects_reorder`, `compare_distinguishes_move_from_add_remove`, `compare_reports_pure_version_change_not_as_move`. Each exercises one corner of the four-axis diff added in v0.2.
- **Migrate**: `migrate_rewrites_library_in_place`, `migrate_preserves_lossless_roundtrip`, `migrate_recursively_rewrites_embedded_frontmatter`, `migrate_is_idempotent_on_v0_2_files`. The third specifically catches the recursive-frontmatter bug that was found during the v0.2 build (Appendix B.5 §5).

The integration suite is 19 tests; combined with the unit suite, v0.2 ships 46 passing tests. If they pass, the system does what it claims.

## Appendix B.5 — Bugs Found During the Build

This is the list of bugs that were caught and fixed during construction. They're useful to read because they show what kinds of mistakes are easy to make and how the architecture either let them through or surfaced them.

**1. Body delimiter level conflated with logical compositional depth (v0.1, fixed in v0.2).**
*Symptom:* `compose([compound, atom])` produced a file that `decompose` couldn't parse — the outer `~~>>` collided with the inner compound's body delimiters.
*Root cause:* the original code used a single `order` integer for two distinct meanings (logical depth label *and* physical delimiter level). A count-based formula picked the logical value, and for mixed inputs that formula did not climb above `max(input.order)`, breaking the escalation rule on the delimiter side.
*Fix:* split into two integers — `body_level` (always `max(input.body_level, default = 0) + 1`, used by the delimiter scheme) and `depth` (the human-friendly compositional-depth label, computed the same way). The count-based formula was retired entirely; both fields now strictly escalate.
*Lesson:* when one variable carries two distinct meanings, the test cases that look complete probably only exercise the diagonal where the meanings agree. Cover the off-diagonal explicitly.

**2. Compare's dispatch ignored kind (v0.1, fixed in v0.2).**
*Symptom:* a hand-authored atom compared against a single-input compound would route to `content_diff`, which then tried to read non-existent fields on one side.
*Root cause:* the dispatch checked the v0.1 `order` field first and only branched on atomicity afterwards, implicitly assuming both inputs would be atom-iff-order-zero. That stopped being true once `compose` could produce single-input compounds.
*Fix:* dispatch on the 2x2 of `(a.is_atom(), b.is_atom())`, with explicit `KindMismatch` error for the diagonal-mismatched cases. With `order` removed in v0.2, this is the single dispatch axis.
*Lesson:* when a function's behavior depends on a relationship between two inputs, write the cross-product as a `match`, not a sequence of `if`s. The compiler will tell you when you've forgotten a case.

**3. Hand-authored mis-kinded files accepted silently (v0.1, fixed in v0.2).**
*Symptom:* under v0.1, a file with `order = 5` and no `composed_of` parsed cleanly; decompose would refuse to operate on it, but only when called. Under v0.2 the equivalent failure mode would be `kind = "atom"` with companion fields populated, or `kind = "compound"` with no recipe.
*Root cause:* the validator only required companion fields when the discriminator implied them. It didn't *forbid* companion fields when the discriminator said they shouldn't be there.
*Fix:* added an `else` arm in the v0.2 validator: if `kind = "atom"`, all five companion fields must be absent; if `kind = "compound"`, all must be present with valid contents. The `CompoundMissingField` error covers the missing-when-required half.
*Lesson:* every "field required when X" check has a dual "field forbidden when not X" that's easy to miss. Spell out both directions.

**4. Create wrote the file before validating, leaving an orphan on bad input.**
*Symptom:* `oovra create --new InvalidName` (not kebab-case) wrote `InvalidName.md` to disk, *then* errored — leaving the unparseable file behind.
*Root cause:* `write` used to be `fs::write` followed by `parse_file` for verification. The verify ran after the disk was already touched.
*Fix:* validate the in-memory serialized form via `parse(&content, path)?` *before* any `fs::write` happens. Disk is only touched if the in-memory form already parses.
*Lesson:* tests that only assert on return values miss side-effect bugs. The fix here required adding a test that does `read_dir` on the target directory after a failed scaffold to verify zero files exist. Side effects deserve side-effect-aware assertions.

**5. Migrate only rewrote outer frontmatter; embedded sub-elements stayed in v0.1 (found during v0.2 work).**
*Symptom:* `oovra migrate ./elements` reported success, but a follow-up `oovra decompose` on any migrated *deep* compound failed with `missing field 'kind'` pointing at an embedded sub-element's frontmatter.
*Root cause:* the first-pass migrate function read each file via `parse_file_with(legacy=true)`, then re-serialized only the outer header. The compound's body still contained v0.1 sub-files verbatim — the legacy loader had accepted them on read but the rewrite never touched them.
*Fix:* a `decompose_with(element, opts)` variant that threads `ParseOptions` into the embedded parsing step, plus a recursive `rewrite_with_body_migration` that legacy-decomposes the body, migrates each sub-element, and re-renders the body from migrated children before re-serializing the outer file.
*Lesson:* when a transformation walks a recursive structure, the test fixtures must include the recursive case explicitly. Shallow fixtures and unit-tests-on-atoms passed; the bug only surfaced when a `body_level = 2` compound was migrated and immediately decomposed. Regression test: `migrate_recursively_rewrites_embedded_frontmatter`.

**6. Sequence-blind structural diff reported reorders as equal (v0.1, fixed in v0.2).**
*Symptom:* composing `[role, safety, tone]` then `[tone, safety, role]` produced two compound files whose rendered prompts differed (the LLM would see different order) but `oovra compare` reported `recipes_equal: true`.
*Root cause:* v0.1's structural diff hashed by ID and reported only `added`/`removed`/`version_changed` — position was discarded.
*Fix:* the v0.2 diff carries `(position, version)` in the by-ID map and adds a fourth axis `moved` for same-id-same-version-different-position. `version_changed` and `moved` are explicitly non-mutually-exclusive: one input can fire on both.
*Lesson:* set-based abstractions over inherently sequential data are a class of silent semantic loss. When the rendering depends on order, the diff must as well — even if it makes the type and the report more complex.

## Appendix C — The Sheet Mapping

Your Google Sheet's four operators map cleanly:

| Sheet | Cells | Oovra | Implementation |
|---|---|---|---|
| JOIN | A1:G1 → H1 (delimited concat) | Compose | `render_body` wraps each input chunk in delimiters and joins |
| SPLIT | B3 → C3:H3 (split on delimiter) | Decompose | walk the body, split at level-N delimiters, parse each chunk as a complete file |
| UNIQUE | B5:C8 → D5 (deduplicate across array) | Library audit (deferred) | hash all `composed_of` IDs across compositions, find rare/common |
| FORWARD-DIFF | B10, C10 → D10 (sequence-aware difference) | Compare | hash-by-id over `composed_of` carrying position, compute added/removed/version-changed/moved |

The structural difference between the Sheet and Oovra: the Sheet operates on string values (and derives types from formulas); Oovra operates on **typed parsed structures** (and derives string outputs from rendering). The format is doing static-typing work that the Sheet has to do dynamically.

## Appendix D — Things Deliberately Out of Scope

- Semver range matching (`^1.0`, `>=1.2`). The current build has exact-string match. The `semver` crate is in `Cargo.toml` for when you need it.
- Dependency resolution — automatic expansion of `requires` references (the field doesn't exist in the schema). Add when you have an actual `requires` graph to walk.
- The `bundle` kind. Reserved for future use.
- Library-wide audits ("which elements are unused", "which appear in 80%+ of compositions"). Easy to add later; nothing forces it now.
- TUI for browsing the library with `ratatui`. Good standalone learning project.
- HTTP API (`oovra serve`). Easy bolt-on once you actually want one.
- Obsidian plugin wrapping the operations. Worthwhile when you've used Oovra for two weeks and know which operations belong on a button.

Resist building these. The discipline is: **ship, dogfood, then plan the next version**.
