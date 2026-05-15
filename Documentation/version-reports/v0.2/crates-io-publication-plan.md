# crates.io Publication Plan

Readiness assessment + step-by-step plan for publishing Oovra v0.2 to [crates.io](https://crates.io). Captured during the v0.2 cycle; **not yet executed** — revisit before the actual publish.

---

## What's already in shape

| Area | Status |
|---|---|
| Tests pass | 46/46 (27 unit + 19 integration) |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo fmt --check` | Clean |
| License | Dual MIT OR Apache-2.0 (both files present at repo root) |
| README | Present, v0.2-current |
| CHANGELOG.md | Present, v0.2 documented |
| Rust edition | 2021 |
| Workspace dependencies | All from crates.io (no `path` / `git` deps to strip) |
| Version | 0.2.0 (pre-1.0 — breaking changes still allowed) |
| `Cargo.toml` basics | name, version, edition, description, license, authors, repository — all set |

## What needs attention before publish

**1. Crate name availability.** `oovra` is unusual enough that it's likely free, but it must be checked at https://crates.io/crates/oovra before anything else. Once taken (even by a 0.1.0 placeholder), no one else can use it.

**2. Cargo.toml is missing six recommended metadata fields:**

| Field | Why it matters |
|---|---|
| `readme = "README.md"` | Without this, crates.io shows no README on the package page |
| `keywords = [...]` | Surfaces the crate in search (≤5 entries; suggested: `prompt`, `llm`, `composition`, `markdown`, `toml`) |
| `categories = [...]` | Must be from the canonical list (https://crates.io/category_slugs); suggested: `command-line-utilities`, `text-processing` |
| `rust-version = "1.XX"` | Declares MSRV; needs determining (probably 1.70 or so, given clap 4 + chrono 0.4 + thiserror 1) |
| `documentation = "https://docs.rs/oovra"` | Stable docs link |
| `homepage` | Optional; can mirror `repository` or omit |

**3. Package size and content.** By default `cargo package` includes everything except `target/`, `.git/`, and `.gitignore`d files. Right now that means the published crate would bundle:

- `oovra_slate.png` (1.17 MB)
- `Fig 1-1 diagram.png` (43 KB)
- `Documentation/` tree (large — five v0.1 demos + v0.2 operator-gamut demo with library snapshots + reference docs + version-reports)
- `elements/` (5 sample atoms)

None of those are needed for someone running `cargo install oovra` or depending on the library. Add an `exclude = [...]` block to drop them. The published crate should be just: `src/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `README.md`, `SCHEMA.md`, `CHANGELOG.md`, `LICENSE-*`. Estimated drop from ~5 MB to ~150 KB.

**4. README image is a relative path.** `<img src="oovra_slate.png">` won't render on crates.io (no relative resolution). Fix by switching to an absolute GitHub raw URL: `https://github.com/crussella0129/oovra/raw/main/oovra_slate.png`. Same fix for any other relative image links.

**5. Library API surface is wide-open.** `lib.rs` does `pub mod create; pub mod decompose; pub mod diff; pub mod element; ...` — every module is publicly accessible. That's fine for a 0.x crate where breakage is expected, but worth a deliberate decision: ship with the broad surface and document "API unstable until 1.0", or tighten now via `pub use` re-exports + `pub(crate) mod`. Recommendation: keep as-is for v0.2; revisit before 1.0.

**6. docs.rs build.** crates.io triggers a docs.rs build on publish. Should pre-verify `cargo doc --no-deps` produces clean output (no warnings, no broken intra-doc links) so the docs.rs page works on day one.

**7. No CI.** Not blocking for crates.io but worth setting up before — a single GitHub Actions workflow (`cargo test` + `clippy` + `fmt --check` on push/PR) catches regressions and shows the green badge on the README/repo.

## Decisions to make

| Decision | Options | Recommendation |
|---|---|---|
| Single crate (lib + bin) or split? | (a) Keep as one crate (current), (b) Publish `oovra` (lib) and `oovra-cli` (bin) separately | (a) — the surface is small enough that splitting adds maintenance overhead without clear benefit |
| Tighten public API now or later? | (a) Ship as-is with broad `pub mod`, (b) Curate via `pub use` before publish | (a) — pre-1.0 is the right window for "API may change"; lock it down before 1.0 |
| Reserve name now or after polish? | (a) Push a 0.2.0 immediately, (b) Polish first (CI, MSRV declaration, doc tests), publish later | (b) — but check name availability *today*, since reservation costs only one publish |

## Implementation plan (sequential — do not start without explicit go-ahead)

### Phase A — Verification (read-only, ~5 min)

1. Visit https://crates.io/crates/oovra (or run `cargo search oovra`) to verify the name is free.
2. Run `cargo package --list` to see the file set that would currently get uploaded (baseline for the `exclude` block).
3. Determine the MSRV: run `cargo build` against successively older toolchains, or read each direct dependency's `rust-version`, and pick the highest.

### Phase B — Cargo.toml + README polish (one commit)

4. Add to `Cargo.toml`:
   - `readme = "README.md"`
   - `keywords = ["prompt", "llm", "composition", "markdown", "toml"]`
   - `categories = ["command-line-utilities", "text-processing"]`
   - `rust-version = "1.XX"` (from step 3)
   - `documentation = "https://docs.rs/oovra"`
   - `exclude = ["Documentation/**", "elements/**", "*.png", ".cargo/**", "target/**"]`
5. Update README image source to absolute GitHub raw URL.
6. Run `cargo package --list` again — confirm only the intended files remain.
7. Run `cargo doc --no-deps` — fix any warnings (broken intra-doc links, missing docs on public items if any).
8. Run `cargo package` — produces `target/package/oovra-0.2.0.crate` locally, verifies the crate builds in the packaged form.
9. Commit + push.

### Phase C — Optional polish (separate commits, optional)

10. Add `.github/workflows/ci.yml` with `cargo test` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`. Confirm green.
11. Add a few doc-tests in `lib.rs` exercising `parse`, `compose`, `decompose`, `compare` so docs.rs shows runnable examples.
12. Add crates.io / docs.rs / CI badges to `README.md`.

### Phase D — Publication (irreversible — needs explicit go-ahead)

13. Get a crates.io API token (one-time, manual; `cargo login` via the user).
14. Run `cargo publish --dry-run` — final read-only confirmation that everything would upload cleanly.
15. Run `cargo publish` — **irreversible**: a published version can only be `yanked` (made unavailable for new dependencies), never deleted. Confirm version number is correct before running.
16. Tag the release: `git tag v0.2.0 -m "v0.2.0" && git push --tags`.
17. Create a GitHub release using the tag with the v0.2 CHANGELOG entry as the release body.
18. Verify docs.rs build succeeded (it auto-triggers on publish; takes ~15 min).

### Phase E — Post-publish (optional)

19. Announce on r/rust "this week in rust", lobste.rs, or wherever relevant.
20. Watch for first issues / PRs; treat them as v0.2.1 / v0.3 input.

---

## What to confirm before starting Phase B

- **Name availability** (Phase A.1). If `oovra` is taken, you'll need a new name everywhere — Cargo.toml, README, build guide, the CLI binary, every doc — and that's a much bigger scope.
- **MSRV** (Phase A.3). You may want to deliberately pin the MSRV higher than strictly necessary (e.g. last stable minus 6 months) to keep your maintenance burden low.
- **Whether optional polish (Phase C) is wanted before publish** or whether shipping the bare minimum first and adding polish in 0.2.1 is acceptable.

---

## Status

**Captured**: 2026-05-14, during the v0.2 docs-refinement cycle.
**Not yet executed**: holding for explicit go-ahead. Phase A is read-only and can be run independently when the user wants a green-light check.
