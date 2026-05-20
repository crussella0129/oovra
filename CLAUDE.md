# CLAUDE.md — `oovra` project rules

These rules apply to any Claude Code session working in this repo and
take precedence over the global `~/.claude/CLAUDE.md`.

## CLI-first, always

**Every capability ships in the CLI first.** The library half (the
`oovra` crate's `lib.rs` tree) is the canonical implementation; the
`oovra` CLI binary is its first consumer; the `oovra-gui` crate
mirrors what the CLI already does.

Concretely, when adding a feature:

1. Add the function(s) to the appropriate module under `src/`. Pure
   Rust, tested by `cargo test -p oovra`.
2. Expose it as a CLI subcommand (or extend an existing one) in
   `src/main.rs`.
3. *Then* surface it in the GUI by calling the same library function
   directly (no subprocess to the CLI).

This rule exists because the user explicitly wants the CLI to keep
working for headless agent boxes that won't ever install the GUI. A
GUI-only feature would leave that surface incomplete. The GUI is a
front-end for humans; the CLI is for humans-on-terminals **and** for
agents. They must reach feature parity at every sprint boundary.

## Autonomy posture

The user expects to leave this project running for hours at a time.
Default to autonomous execution; the only acceptable reasons to stop
mid-stream are:

- **A symmetry-breaking architectural decision** — a real fork where
  going down the wrong path is expensive to undo (data layout
  changes, public-API renames, framework swaps, multi-crate
  re-orgs). Routine choices (file naming, error-message wording,
  dependency patch bumps, how to factor a function) are not
  symmetry-breaking — decide and proceed.
- **A visual heartbeat** — a moment when the user genuinely needs to
  look at or interact with a UI change before the next step makes
  sense. Launch the app with `cargo run -p oovra-gui` in the
  background and continue working; the running window is the
  heartbeat. Don't block on confirmation unless the *next* sprint
  depends on something the user has to physically click.

Things that are explicitly NOT acceptable reasons to stop:

- Asking permission to `cd` somewhere or run a shell command.
- Asking permission to wait on CI / a long build / a background task
  — start it in `run_in_background = true`, do other work or use
  Monitor with an `until` loop, then keep going when it finishes.
- Asking which of two roughly-equivalent technical choices to take.

## Sprint methodology

Work is organized in numbered sprints under `sprints/sN/` per the
methodology already in flight (Research → Plan → Build → Test →
Close). Agent task tracking lives in `agent-tasks/agent-tasks.md`
(active) and `agent-tasks/completed-tasks.md` (log).

A sprint is closed when its `sprint-tests/test-report.md` records
PASS for all acceptance criteria. Roll into the next sprint
immediately; don't wait for explicit confirmation. The user reviews
asynchronously.

## Workspace and install paths

`oovra` is a Cargo workspace with two installable artifacts:

- **CLI only** — `cargo install --path .` (root package). For
  headless boxes and agent hosts.
- **Full GUI** — `cargo install --path gui` (the `oovra-gui`
  package; pulls the `oovra` library as a path-dep).

Don't break the CLI-only install path. Workspace member changes that
add deps must keep `cargo install --path .` lean (no eframe / wgpu /
winit downloads). Verify with `cargo build -p oovra` for fast
sanity.

## Commit hygiene

Commit work in coherent logical units as you finish them — don't let
the working tree drift across sprints unless an active step demands
it. Don't push without an explicit instruction from the user (push
is the irreversible step; commits are local and trivially revertible).
