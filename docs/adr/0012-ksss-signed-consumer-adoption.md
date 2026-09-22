# ADR-0012: Consume the approved KSSS runtime locally

Status: Accepted for this owner-directed local adoption branch.
Date: 2026-09-20
Owners: KaspaPulse repository owner and KGW maintainers.
Supersedes: none. Extends ADR-0011; leaves ADR-0010 runtime topology unchanged.

## Context
Owner reports unresolved network and bridge behavior and requests KSSS adoption
as used by Maki and Gov Forms. Repeated broad audits and stale success evidence
must not substitute for proof of actual packaged application behavior.

## Decision
Consume the signed v1.2.0 runtime pinned to central source
967ed5068947a39961d5d5cc483ef65d25a61059 using a thin KGW adapter.
Reuse Gov Forms' exact trust material and source comparison evidence where the
signed bytes are identical; reverify cryptographic acceptance locally.
Run the existing central resolver, classifier, schemas and knowledge logic from
the verified archive. Normal evaluation remains offline.
Keep KGW-specific applicability, threat boundaries and runtime receipt contract.
Risk floors survive profile selection. Source/tree/artifact/environment changes
invalidate evidence. Learning is separate from service restoration.

## Alternatives
A private KGW policy engine would drift from the approved runtime.
A central checkout on every invocation adds an unnecessary availability dependency.
Copying Gov Forms' application checks would miss KGW process and network behavior.

## Consequences
One reusable consumer model and a small local gate reduce repeated work.
The repository now carries signed artifacts and adapter maintenance obligations.
Receipt validation checks identity, completeness and file hashes; an operator
must still produce and review actual packaged application evidence.
Governance PASS does not certify application health or authorize publication.

## Validation and related records
See .security/ksss/README.md, its contract tests, trust/evidence/v1.2.0/,
and docs/handoff-ledger/2026-09-20-kgw-ksss-v1.2-adoption.md.
Local cryptographic acceptance and 24 focused tests pass. No live runtime claim.
No GitHub PR or deployment is part of this decision.

## Change history
2026-09-20: recorded owner-directed local repository adoption.
