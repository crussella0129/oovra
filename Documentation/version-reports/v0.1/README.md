# v0.1 — Versioned Artifacts

This folder holds artifacts that are **version-specific to v0.1**: planning documents, assessment reports, and other materials that pertain to a particular release rather than to the project in general.

The rest of `Documentation/` (the `reference/`, `demos/`, `build-guide.md`) is version-current — it always describes the latest shipping version. This folder is the **historical record** for v0.1 specifically.

## Contents

- [`version-reports/`](./version-reports/) — three planning/assessment reports written during v0.1: v0.2 scoping, oovra-server feasibility, and an efficacy rating of v0.1 itself.

## When v0.2 ships

When v0.2 ships:

- This `v0.1/` folder stays in place as the historical record
- A new `v0.2/` folder is created for v0.2-era artifacts
- The version-current docs (`reference/`, `demos/`, `build-guide.md`) get updated in place to reflect v0.2

This keeps the per-version artifacts separated and easy to browse — you can read "what was being thought about during v0.1" without context-switching to the current state.
