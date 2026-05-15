# Oovra v0.3 Scoping Report

A snapshot analysis of v0.2's surface, captured during the v0.2 cycle, with concrete proposals for v0.3. The scope contract — revisit when v0.3 work begins.

## Guiding principles

These constrain everything below.

1. **Don't add features that are shortcuts for what already works.** Every proposal is checked against "can this be done with existing operators?" If yes, it's documented as a discovered usage pattern, not an addition.
2. **No new top-level commands unless the category is genuinely different.** `create`, `compose`, `decompose`, `compare`, `migrate` cover the four-operator algebra plus the v0.1→v0.2 migration. New verbs are added only when no existing verb can absorb the operation — e.g. **acquisition** (pulling from a remote source) is categorically different from **transformation** (operating on local files).
3. **Bias toward extensibility.** One new flag that accepts many input formats beats N specialized flags. `--import <path>` dispatching on extension beats `--import-csv`, `--import-tsv`, `--import-json`.
4. **Keep the agent-facing surface predictable.** LLM agents author these files; an unstable surface defeats the purpose. Pre-1.0 means breakage is allowed, not encouraged.

---

## Part 1 — Lessons from v0.2 (carry forward)

- **Discriminator-as-string beats discriminator-as-integer for path-independence.** `kind = "atom" | "compound"` is a property of the artifact; `order = N` was path-dependent (same atoms via different bracketings produced different N). The lesson generalizes: discriminators that are functions of *content* survive refactoring; discriminators that are functions of *history* don't.
- **Sequence-awareness matters when sequence affects rendering.** v0.1's set-based structural diff was a silent semantic loss. Whenever a data structure's *order* affects downstream behavior, the diff over it must report position changes.
- **Recursive data structures need recursive test fixtures.** The migrate-only-rewrites-outer-frontmatter bug passed every shallow test and failed the moment a deep compound was migrated and decomposed. Lesson: fixtures must include the recursive case explicitly, not infer it from the flat case.
- **The self-extracting-archive property is load-bearing.** Compounds embed full sub-files, not just IDs. `decompose --full` works without library access. Preserve this in every new feature — a compound is its own complete artifact.
- **Two integers, intentional decoupling.** `body_level` (parser-facing) and `depth` (human-facing) are numerically identical for every valid v0.2 compound. They're separate fields so a future renderer can change the delimiter convention without renaming the user-visible depth metric. Don't collapse them.
- **The build guide is a derivative of working code, not of spec.** v0.2's guide was rewritten line-by-line from the v0.2 source after the migration shipped, and it caught two spec/code drifts (one in the validator, one in Appendix A). Keep this discipline: any v0.3 spec changes also get the build guide refreshed against the post-merge source.

---

## Part 2 — Proposed flag-level additions to existing operators

Everything in this section slots under an existing top-level command. No new verbs.

### `create` — bulk import + multi-create

#### `--import <path>` (with optional `--format <csv|tsv|json|jsonl>`)

**Purpose**: turn a tabular file into N atoms in one invocation. Importing a tabular file to make `.md` files is `create` at scale — the natural home is under `create`, not a new verb.

**Format dispatch**: by file extension when `--format` is omitted; explicit `--format` overrides. v0.3 ships CSV, TSV, JSON, JSONL. XLSX is deferred (extra dependency; revisit when there's demand). Adding a future format is a new arm in the `match` — no flag-multiplication.

**Canonical CSV format** (proposed; finalize before implementation):

| Column | Required? | Notes |
|---|---|---|
| `id` | yes | kebab-case |
| `name` | yes | non-empty |
| `version` | yes | semver |
| `meta` | no | default `""` |
| `body` *or* `body_file` | yes (one of) | inline body string, or path to a sibling `.md` |

Body input has two modes (mutually exclusive per row):

- **`body` column** — inline body string. Multi-line bodies via standard CSV `"..."` quoting.
- **`body_file` column** — path to a sibling Markdown file, resolved relative to the CSV's directory. Cleaner for long prompts and lets authors keep bodies in editor-friendly `.md` files while batching the metadata.

Header-detection rule: the CSV must have either `body` or `body_file` as a column header, not both. The header on the first row picks the mode.

**Failure policy**: each row is validated against the same `kind = atom` validator as `--new`. Default behavior on failure: collect errors, continue with valid rows, report all failures at the end. `--strict` would abort on first failure (mirroring `cargo build`).

**JSON / JSONL variants**: array (or one record per line) of `{id, name, version, meta, body}`. Same validation, same failure policy.

#### `--new <ID1> [<ID2> ...]` accepting multiple IDs

Today `--new` takes one ID. Multiple-ID acceptance is a pure convenience for ad-hoc batches that aren't worth writing a CSV for. Each ID scaffolds an atom with default metadata; same TODO body placeholder.

### `compose` — JSON output + input-from-file

#### `--format json` on compose

Today `compose` prints a human-readable line. JSON output gives scripts (and the future Electron frontend) something structured to parse. Same struct (`ComposeReport`?) derives `Serialize`. Symmetric with `decompose --format json` and `compare --format json`.

#### `--from <file>` for input list

Long compose invocations are painful on the command line. `--from inputs.txt` reads whitespace-separated IDs (one per line, with optional `id@version` pins) and uses them as the positional input list. Convenience; no semantic change.

### `decompose` — partial recursion + tree format

#### `--depth <N>`

Today `--full` is all-or-nothing (recurse to atoms) and the default is one-level. `--depth 2` peels off two levels and leaves the rest wrapped. Useful for medium-deep trees where you want a glimpse without the full unroll.

#### `--format tree`

ASCII tree view of the recipe with branch glyphs (`├──`, `└──`, `│`). Complements `human` (list) and `json` (machine). Useful for visualizing deep compositions in a terminal without `--full`-ing to disk.

### `compare` — library-aware + ignore-fields

#### `--against-library <dir>`

Today `compare A B` is path-to-path. `compare ./drift.md --against-library ./elements` reads B's `id` from the file, looks up the current library version, and compares against that. Useful for drift detection ("did someone hand-edit this compound? has the library moved past it?").

#### `--ignore <field>`

Skip a metadata field in content-diff output. `--ignore meta` is the common case (meta is human-facing description; rewording it shouldn't flag as a diff). Can be repeated: `--ignore meta --ignore name`. The body diff is unaffected.

#### `--library <dirA> <dirB>` for library-wide diff (borderline)

Pairwise compare across two library directories. Output: per-ID add/remove/version-change/content-change summary. Borderline because the same can be achieved with a shell loop calling single-file compare — consider the ergonomics-vs-creep tradeoff before implementing. Open decision (Part 6).

---

## Part 3 — Justified new top-level command

### `fetch` — acquisition (categorically different from transformation)

**Justification**: every existing command operates on files already in the local library. `fetch` *acquires* elements from a remote source — URL, git repo, or local directory (for symmetry). This is a different category of operation: network I/O, source authentication, conflict resolution against local elements. No existing verb naturally absorbs it.

**Initial surface (v0.3 MVP)**:

```
oovra fetch <source> [--library <dir>] [--into <subdir>] [--overwrite|--namespace <prefix>]
```

Three source types:

- **Git repo**: `oovra fetch git@github.com:user/prompt-pack.git`
- **HTTP archive**: `oovra fetch https://example.com/registry/coding-agents.tar.gz`
- **Local directory**: `oovra fetch file:///path/to/local-library/` — symmetric with remote, useful as a thin wrapper over `cp -r` that also validates the incoming files

**Conflict policy**: refuse if any incoming ID collides with an existing library ID, unless `--overwrite` (replace) or `--namespace <prefix>` (rewrite incoming IDs to `<prefix>-<id>`) is passed.

**Deferred to v0.4+**:

- Registry / discovery service (find packs by topic, ratings, etc.)
- Signed releases / authenticity verification
- Lockfile for reproducible fetches

v0.3 ships pull-by-URL only. A real registry can come once demand exists.

---

### Considered and rejected for new-top-level status

- **`audit` / `stats`** — aggregate library introspection (unused atoms, version frequencies, average compound depth). Sounds like a new verb but is achievable as `decompose --library <dir> --format json | jq ...` once `decompose` accepts library scope. **Decision**: do not add as a new top-level; instead, propose `decompose --library <dir>` as a flag operating over a whole directory in a future minor (probably v0.3.x or v0.4).
- **`validate`** — every operator already validates internally. `oovra compose --text <id>` is the staged validation path. Adding `validate` duplicates.
- **`serve`** — HTTP / network service. Truly different category (process model), but the v0.1 feasibility report recommended deferral, and v0.3 has enough other scope. **Decision**: continue to defer to v0.4+. Track but don't ship in v0.3.

---

## Part 4 — Cleanup / maintenance (no new features, but v0.3-shipped)

These are debts called out during v0.2 work; v0.3 is the natural window.

1. **Remove `--legacy` flag and `LegacyHeader`** (announced as deprecated in v0.2 docs). Users who haven't migrated by v0.3 run `oovra migrate` first.
2. **Accept `.oov` as an alternate extension.** Keep `.md` as canonical (Obsidian compatibility is load-bearing); add `.oov` as an accepted file extension for users who want stronger format separation. One-line change in `library.rs` and `migrate.rs` walkers.
3. **Resolve duplicate-id limitation in `diff.rs`.** Today the structural-diff HashMap collapses duplicate IDs (second occurrence wins). Switch to an LCS-based diff that handles duplicates correctly. Called out in `src/diff.rs` module docs.
4. **Library-wide cycle detection in `compose`.** Today a compound *can* indirectly reference its own ID through a child compound (the recipe is one-level; the cycle only manifests at decompose time). Detect at resolution time and refuse cleanly.
5. **Doc tests in `lib.rs`** for `parse`, `compose`, `decompose`, `compare`. Improves the docs.rs page and catches API breakage. Useful regardless of crates.io publication timing.
6. **API surface review.** If v0.3 ships alongside crates.io publication, decide whether to lock `pub mod` down to `pub use`. See [`crates-io-publication-plan.md`](./crates-io-publication-plan.md).

---

## Part 5 — Out of scope for v0.3 (track separately)

- **Electron frontend** — separate project, separate language stack, separate repo. v0.3 *enables* it by adding `--format json` to compose and `create --import` for CSV-based scaffolding, but the frontend itself is its own roadmap.
- **Tauri port** — strictly after the Electron app reaches functional maturity (per CLAUDE.md preferences).
- **crates.io publication** — has its own plan document ([`crates-io-publication-plan.md`](./crates-io-publication-plan.md)); can happen during v0.2.x patch or alongside v0.3 ship. Decoupled from v0.3 feature work.
- **Semver range matching in `composed_of`** (`^1.0`, `>=1.2`) — listed as deferred in the build guide. The `semver` crate is already a dependency for when this is needed. Not v0.3 unless a real use case surfaces.
- **`oovra serve`** — track for v0.4+.

---

## Part 6 — Open decisions before implementation

1. **Canonical CSV format for `create --import`**:
   - Inline `body` column with CSV-quoted multi-line strings?
   - `body_file` column referencing sibling `.md` files?
   - Both, mutually exclusive per row (recommended above)?

2. **Library-wide diff (`compare --library a/ b/`)** — ship in v0.3 or defer to v0.4? It's ergonomics over a shell loop, not a new capability.

3. **`decompose --library <dir>`** for aggregate introspection — ship in v0.3 alongside the audit-like use cases, or defer to v0.4?

4. **Order of work**: cleanup-then-features, or features-then-cleanup? Cleanup items (`--legacy` removal, duplicate-id LCS, cycle detection) are smaller and lower-risk. Features (`create --import`, `fetch`) are user-facing and exciting. **Recommendation**: clean up first (one PR per item, mergeable independently), then features. Mirrors the v0.2 phased approach.

5. **Versioning policy on v0.3**: bumping to v0.3.0 will be another breaking change (`--legacy` removal). If keeping the option to publish v0.2.x patch releases on crates.io, lock in v0.3 timing before any crates.io push.

---

## Summary table

| Proposal | Type | Where it lives |
|---|---|---|
| `create --import <path>` (CSV/TSV/JSON/JSONL) | new feature | flag on `create` |
| `create --new` accepting multiple IDs | convenience | flag on `create` |
| `compose --format json` | new feature | flag on `compose` |
| `compose --from <file>` | convenience | flag on `compose` |
| `decompose --depth <N>` | new feature | flag on `decompose` |
| `decompose --format tree` | new feature | flag on `decompose` |
| `compare --against-library <dir>` | new feature | flag on `compare` |
| `compare --ignore <field>` | new feature | flag on `compare` |
| `compare --library <a> <b>` | new feature (borderline) | flag on `compare` |
| `oovra fetch` | new verb | new top-level (acquisition category) |
| `--legacy` and `LegacyHeader` removal | cleanup | removal |
| `.oov` extension acceptance | cleanup | parser/walker change |
| Duplicate-id LCS in `diff.rs` | cleanup | algorithm replacement |
| Library-wide cycle detection in `compose` | cleanup | resolver change |
| Doc tests in `lib.rs` | cleanup / docs | new tests |
| API surface review | cleanup | gated by crates.io decision |

---

## Snapshot date

Captured 2026-05-14, during the v0.2 cycle. Revisit before v0.3 work begins. This document is the scope contract.
