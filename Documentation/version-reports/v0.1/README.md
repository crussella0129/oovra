# v0.1 — Version Reports

This folder is the **historical record** for v0.1: planning artifacts, assessments, and the v0.2 migration specification — captured while v0.1 was the current shipping state.

The version-current docs (`reference/`, `demos/`, `build-guide.md`) live elsewhere under `Documentation/` and always describe the latest shipping version (v0.2 as of this snapshot). This folder is **frozen at v0.1**.

## Contents

| File | Purpose |
|---|---|
| [`SPEC-v0.2.md`](./SPEC-v0.2.md) | The v0.2 migration specification (Draft 3). The contract that v0.2 was built against. |
| [`v0.2-scoping.md`](./v0.2-scoping.md) | Original scope contract for v0.2: what to add, what to defer. |
| [`v0.1-efficacy-rating.md`](./v0.1-efficacy-rating.md) | Honest assessment of v0.1 as a shipped tool (rating: 7.4/10). |
| [`v0.1-ceiling-analysis.md`](./v0.1-ceiling-analysis.md) | Twelve structural limits and what it would take to push the ceiling. |
| [`oovra-server-feasibility.md`](./oovra-server-feasibility.md) | Design exercise for an HTTP/network service (deferred). |
| [`README2.md`](./README2.md) | Detailed reading-order guide for the four assessment reports — kept as a separate index after the v0.2 folder reorg. |

See [`README2.md`](./README2.md) for an annotated table with reading order and how the four assessment reports relate to one another. SPEC-v0.2.md is the v0.2 contract proper — read it before reading the v0.2 code.

## What changed when v0.2 shipped

- The numeric `order` field was replaced by `kind = "atom" | "compound"`. See SPEC-v0.2.md §1.
- The structural diff in `oovra compare` became sequence-aware (a `moved` axis was added). See SPEC-v0.2.md §6.
- `oovra migrate` was added as a new subcommand for in-place v0.1 → v0.2 library conversion. See SPEC-v0.2.md §3.5.
- The license was changed from GPL-3.0 to MIT OR Apache-2.0 dual.

The full v0.1 → v0.2 change list is in [`../../../CHANGELOG.md`](../../../CHANGELOG.md).

## Snapshot date

All reports here were generated against **Oovra v0.1.0**. See the git history for exact dates.
