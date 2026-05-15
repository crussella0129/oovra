# v0.2 Edge-Case Test Findings

End-of-v0.2 testing pass: 8 categories, ~30 scenarios run against the release binary (`target/release/oovra.exe`, oovra 0.2.0). Fixtures and capture logs lived in `target/edge-tests/` and were cleaned up after.

This is the **triage** of behaviors observed. Each entry lists what was tested, what was observed, and a triage tag: **fix-in-v0.2.1** (real bug, small fix), **defer-to-v0.3** (larger change or already on roadmap), **document-as-designed** (intentional, just needs to be visible), or **reassurance** (works correctly — no action).

---

## Existing coverage (before this pass)

- **27 unit tests** across `src/` cover schema validators, parse/serialize round-trip, kind-aware validator dispatch, legacy loader mapping, body_level / depth math, delimiter formula.
- **19 integration tests** in `tests/end_to_end.rs` cover happy-path Create, Compose (atoms-only, mixed-kind, deep), Decompose (one-level, --full, deep tree), Compare (all four diff axes individually and combined), and Migrate (rewrite, idempotency, recursive body rewrite, lossless round-trip).

Together they cover every operator's golden path and the known v0.1 → v0.2 regressions. They do **not** systematically cover edge cases around encoding, malformed input, library hygiene, or pathological compose inputs — that's what this pass adds.

---

## Findings — triaged

### Real bugs found (fix in v0.2.1)

#### 1. `create --new <existing-id>` silently overwrites existing files — **data-loss**

**Severity**: high. A user's hand-tuned atom can be replaced by the scaffold TODO placeholder without warning.

**Reproduction**:

```bash
oovra create --new important --library ./lib   # weeks of work in important.md
oovra create --new important --library ./lib   # ← silently overwrites with TODO body
```

The file gets replaced with the scaffold defaults: `name = "important"`, `meta = ""`, body = TODO comment. The previous content is gone.

**Why**: `create::scaffold` calls `element::write` directly, which has no exists-check. `--label` already has the right guard (`AlreadyLabeled` error unless `--force`) — `--new` is missing the parallel.

**Fix**: refuse the operation if `<library>/<id>.md` already exists, unless `--force` is passed. Mirror `--label`'s guard exactly.

**Tag**: **fix-in-v0.2.1**.

---

#### 2. `decompose --full <atom>` succeeds; `decompose <atom>` errors — **inconsistency**

**Severity**: medium. Behavioral inconsistency surprises users who expect either mode to refuse atom input.

**Reproduction**:

```bash
oovra decompose ./atom.md           # → "Cannot decompose atom 'x'. Atoms have no recipe."
oovra decompose --full -o ./out ./atom.md   # → succeeds, writes out/x/x.md
```

**Why**: `decompose::report` checks `is_atom()` and returns `CannotDecomposeAtom`. `decompose::decompose_full` calls `write_recursive`, which writes the element then returns early on `is_atom()` — never errors. The one-level mode goes through `report`; the recursive mode does not.

**Fix**: make `decompose_full` also refuse on atom input (the operation is trivially "copy the atom to out_dir/atom-id/atom-id.md", which a user would do with `cp`). Consistent with `decompose <atom>`.

**Tag**: **fix-in-v0.2.1**.

---

#### 3. Stale v0.1 vocabulary in `--re-render` error — **cosmetic but visible**

**Reproduction**:

```bash
oovra compose --library ./lib --re-render ./atom.md
# → Error: --re-render target is order 0 and has no composed_of
```

The message says "order 0" — v0.1 vocabulary. The whole codebase migrated to `kind = "atom" | "compound"` but this `anyhow!` string in `main.rs::run_compose` was missed.

**Fix**: change message to `--re-render target is an atom and has no composed_of`.

**Tag**: **fix-in-v0.2.1**.

---

#### 4. `Library::load` aborts on any non-Oovra `.md` file in the directory — **ergonomic wart**

**Severity**: medium. Any plain Markdown file (a README, an `_index.md`, an authoring note) in the library directory aborts the entire library load. The migrate command's `looks_like_oovra_file` pre-check already solves this for the same kind of walk; `Library::load` doesn't use it.

**Reproduction**:

```bash
echo "# Notes" > ./elements/README.md
oovra compose --library ./elements --text role-statement
# → Error: Missing opening '+++' delimiter on line 1 of ./elements/README.md
```

**Fix**: in `library.rs::Library::load_with`, apply the same `looks_like_oovra_file` pre-check that `migrate.rs::migrate_library` already uses. Skip non-Oovra `.md` files with a debug-level skip message; only abort on `.md` files that *claim* to be Oovra (i.e. start with `+++`) but are malformed.

**Tag**: **fix-in-v0.2.1** (small change, immediate UX win) — or alternatively defer to v0.3 if we want to consolidate library cleanup work.

---

#### 5. "File not found" for missing directories — **wording inconsistency**

**Reproduction**:

```bash
oovra compose --library /does/not/exist --out-id x -o ./x.md alpha
# → Error: File not found: /does/not/exist
```

The target is a directory, not a file. `OovraError::FileNotFound` is used both for missing files (parse) and missing directories (Library::load), with the same "File not found" prefix.

**Fix**: either split into `FileNotFound` and `DirectoryNotFound`, or change the message to `Path not found`. Smaller change is the message rewording.

**Tag**: **fix-in-v0.2.1**.

---

### Defer to v0.3 (larger changes or already scoped)

#### 6. Indirect cycles in `composed_of` graph are allowed

**Observed**: building compound A that references B, then B that references A, is permitted. Each compose call embeds a *snapshot* of its inputs, so the body tree is finite even though the library naming graph has a cycle.

**Triage**: this is on the v0.3 scoping list (Part 4, item 4: "Library-wide cycle detection in compose"). **Not urgent** because there's no infinite-recursion risk — embedded snapshots make cycles physically impossible in the body tree. But it's a real ergonomic issue: a user reasoning about "what does cyc-b currently produce?" must remember which library state was active when it was last composed. v0.3 fix as planned.

**Tag**: **defer-to-v0.3** (already scoped).

---

#### 7. Duplicate IDs in `compose` input list silently produce duplicated body content

**Observed**: `oovra compose alpha alpha beta` succeeds, produces a compound with `composed_of: [{alpha}, {alpha}, {beta}]`, and a body containing the alpha file embedded twice. `compare`'s duplicate-id limitation (HashMap collapse) means subsequent diffs over this compound's position changes for `alpha` won't detect moves correctly.

**Triage**: known limitation, documented in `src/diff.rs` module docs and v0.3 scoping (Part 4, item 3: "Resolve duplicate-id limitation in diff.rs"). The compose-side acceptance is the right default — there are legitimate use cases (e.g. asking the model the same thing twice in different contexts). The diff-side handling is what's worth fixing.

**Tag**: **defer-to-v0.3** (already scoped).

---

#### 8. BOM-prefixed files are rejected

**Observed**: a UTF-8-with-BOM file (`EF BB BF` before `+++`) is rejected with `MissingOpenDelimiter` because line 1 reads as `<BOM>+++`. The error message correctly shows the BOM char.

**Triage**: defensible strictness — users who hit this are usually using a Windows editor (Notepad's "UTF-8 with BOM") and can switch to "UTF-8" (no BOM). But it would be a one-line fix in `split_frontmatter` to strip a leading `\u{FEFF}` from content before checking line 1, and BOMs are a recurring footgun on Windows.

**Tag**: **defer-to-v0.3** (small, but no urgency — add as a v0.3 cleanup item).

---

### Document as designed (no code change needed)

#### 9. One malformed Oovra file aborts the whole `Library::load`

**Observed**: a file claiming to be Oovra (starts with `+++`) but failing validation aborts the entire library load. This is intentional fail-loud behavior — a library is meant to be a consistent collection, and silent skipping would hide real authoring errors.

**Triage**: this is the right design and is explicitly documented in `library.rs`. The thing to fix is the *other* class (finding 4 — non-Oovra `.md` files should be silently skipped, the same way `migrate` already does). Once finding 4 lands, the remaining strict behavior is exactly what you want.

**Tag**: **document-as-designed**.

---

#### 10. `create --new` for an `id` whose file doesn't exist still happily creates it even if a *different* file in the library has the same `id`

**Not tested explicitly** but worth noting: `create::scaffold` writes to `<library>/<id>.md`. If `<library>/elsewhere.md` already has `id = "<id>"` in its frontmatter, the new file is created, and the *next* `Library::load` will fail with `DuplicateId`. The current design surfaces the duplicate at load time, not at create time.

**Triage**: this is the same load-time-vs-create-time tradeoff. Detecting at create time would require loading the whole library on every `create` invocation. Probably not worth it; the `DuplicateId` error on next load is actionable.

**Tag**: **document-as-designed**.

---

### Reassurances (things that work correctly)

These were tested and behave well. Listed for completeness so the next test pass can skip them.

| Behavior | Verdict |
|---|---|
| CRLF line endings (`\r\n`) | Parsed correctly; body recovered cleanly |
| Non-ASCII unicode in body (CJK, RTL Arabic, emoji, math symbols) | Round-trips unchanged through compose/decompose |
| Trailing whitespace on `+++` lines | Tolerated (`trim_end` in splitter) |
| TOML-special characters in `meta` (`"`, `[`, `=`) | Properly escaped on serialize, recovered on parse |
| 50KB single-line body | No performance issue; full content recovered via `--text` |
| Bare-string `kind = atom` (unquoted) | Rejected with TOML caret-pointer error |
| Single-input compose | Produces valid body_level=1 compound; decompose works |
| Empty body (whitespace-only) | Rejected with `EmptyBody` error |
| Leading whitespace on `+++` line | Rejected with clear "Expected '+++', got '   +++'" |
| Compare a file with itself | `recipes_equal: true` (compounds) or empty diff (atoms) |
| Compare two compounds with identical recipes but different timestamps | `recipes_equal: true` (the architectural payoff) |
| Compare two atoms with identical body but different metadata | Three `field_changes`, body diff empty, `bodies_equal: true` |
| `decompose --full` re-run into same output dir | Files overwritten in place; no orphans |
| Decompose corrupted compound (missing close, extra open) | Clear `BodyParse` error with body_level context |
| Migrate empty directory | Clean 0/0/0 summary |
| Migrate directory with mixed Oovra and non-Oovra `.md` | Oovra files migrated, non-Oovra skipped with "not-oovra" tag |
| Migrate a body_level=2 compound | Recursively rewrites every embedded frontmatter; subsequent v0.2-strict `decompose` succeeds |
| Migrate already-v0.2 library | Idempotent (re-serializes but content unchanged) |
| `--re-render` with version-pin mismatch | Clear `VersionMismatch` error |
| `--re-render` with missing input | Clear `ElementNotFound` error |
| `--library` pointing at a file (not directory) | Clear `NotADirectory` error |
| clap rejects `--output` + `--text` together | Clean argument-conflict error |
| clap rejects `--new` + `--label` together | Clean argument-conflict error |
| Subdirectories within library | Walked recursively by `walkdir` |

---

## Proposed v0.2.1 patch release

Findings 1, 2, 3, 4, 5 are all small, localized fixes. Together they make a reasonable v0.2.1 patch:

- **#1**: add exists-check + `--force` flag to `create::scaffold` (~10 lines + test).
- **#2**: add `is_atom()` guard to `decompose::decompose_full` (~5 lines + test).
- **#3**: change `--re-render` error string from "order 0" to "an atom" (1 line).
- **#4**: add `looks_like_oovra_file` pre-check to `Library::load_with` (~5 lines + test).
- **#5**: reword `FileNotFound` to `Path not found`, or split the variant (~5 lines).

Total: probably one focused afternoon. Each is independent and could ship as its own PR if desired. Tests for #1, #2, #4 should be added to `tests/end_to_end.rs`.

Decisions to make before starting v0.2.1:
- For **#1**, do we want `--force` semantics identical to `--label`'s (error without `--force`, replace with `--force`), or do we want `--new` to error and require the user to either `--force` or delete the file first?
- For **#4**, do we want `Library::load` to log skip events (e.g. to stderr), or skip silently? Migrate's current behavior is silent in the summary's `skipped` list; library doesn't return such a list.

---

## What's NOT yet tested

Worth flagging for a future pass:

- Path edge cases: symlinks, very long paths, paths with spaces, Windows reserved names (`con.md`, `aux.md`)
- Concurrent access: two `oovra` invocations writing to the same library at once
- Filesystem read-only: a library on a read-only mount
- Pathological compound depths (10+ levels): performance and correctness
- Inputs that include the output's own ID directly (`oovra compose --out-id x x` where `x` exists in library)
- `compose --text` with `--re-render` (probably a clap conflict but not verified)
- All operators when stdin/stdout is a pipe vs a terminal (ANSI color handling)
- Negative `body_level` values in v0.1 legacy files (should fail validation cleanly)

These can be queued for a follow-up edge-case pass.

---

## Snapshot date

Captured 2026-05-14, end of v0.2 cycle, on Windows 11 (gitbash + PowerShell). Release binary `target/release/oovra.exe` from commit `57e6a1f`.
