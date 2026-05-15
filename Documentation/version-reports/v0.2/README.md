# v0.2 — Version Reports

This folder holds artifacts that are **version-specific to v0.2**: planning documents, feasibility studies, publication plans, and other materials that pertain to the v0.2 release cycle rather than to the project in general.

The rest of `Documentation/` (`reference/`, `demos/`, `build-guide.md`) is version-current — it always describes the latest shipping version. This folder is the **historical record** for v0.2 specifically.

## Contents

| File | Purpose |
|---|---|
| [`crates-io-publication-plan.md`](./crates-io-publication-plan.md) | Readiness audit + step-by-step plan for publishing v0.2 to crates.io. Captured but not yet executed. |
| [`forward-looking-analysis.md`](./forward-looking-analysis.md) | Forward-looking analysis from the v0.2 cycle: scope contract for v0.3, including flag-level additions to existing operators, one new top-level command (`fetch`) justified by the no-new-top-levels rule, cleanup items, and explicit out-of-scope tracking. |
| [`edge-case-test-findings.md`](./edge-case-test-findings.md) | End-of-v0.2 testing pass: 30 scenarios across 8 categories. Triages five real bugs (5 small fixes for a v0.2.1 patch), confirms three known limitations, and lists 23 behaviors that work correctly. |

More artifacts may be added as v0.2 planning continues (frontend feasibility, file-extension followups, etc.).

## See also

- [`../v0.1/`](../v0.1/) — the v0.1-era planning artifacts (the v0.2 SPEC itself lives there as the contract v0.2 was built against)
- [`../../../CHANGELOG.md`](../../../CHANGELOG.md) — what v0.2 actually shipped
- [`../../reference/`](../../reference/) — the v0.2 reference docs
