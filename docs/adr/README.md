# Architecture Decision Records

`docs/adr/` is the canonical location for new Architecture Decision Records (ADRs) that capture consequential, durable decisions. ADRs explain why a decision exists, its alternatives and trade-offs, and the evidence used to accept or supersede it.

## When to Create an ADR

Create an ADR for decisions such as runtime/process topology, security boundaries, major persistence/API/deployment strategy, compatibility policy, or another change that is expensive to reverse. Do not create ADRs for routine fixes, formatting, dependency bumps, or implementation details that Git history already explains.

## Numbering

- Use four-digit monotonically increasing identifiers: `NNNN-short-title.md`.
- The repository already contains historical `ADR-0010` at `docs/architecture/adr-0010-same-exe-parallel-self-worker-runtime.md`; it is retained in place to avoid breaking references.
- New canonical ADR numbering therefore continues from `0011`.

## Lifecycle

Allowed statuses:

- `Proposed` — under active evaluation.
- `Accepted` — current decision.
- `Deprecated` — discouraged/obsolete but not replaced by one specific ADR.
- `Superseded` — replaced by a newer ADR.

Do not delete an ADR merely because the decision changed. Mark it `Superseded` and link both directions between the old and new records.

## Required Content

Each ADR should include:

- status and date;
- owners;
- supersedes/superseded-by links when applicable;
- context;
- decision;
- alternatives considered;
- positive and negative consequences;
- validation/evidence;
- related files and PRs/issues/commits;
- change history when lifecycle metadata changes.

## Index

- `docs/architecture/adr-0010-same-exe-parallel-self-worker-runtime.md` — **Accepted** — historical location retained for compatibility.
- `0011-repository-native-project-continuity.md` — **Accepted** — canonical project-state/continuity and source-of-truth model.
