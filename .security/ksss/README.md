# KGW KSSS adoption

KGW consumes the approved KSSS v1.2.0 runtime. This is repository adoption,
not an assertion that the application, networks or bridges work.
Application runtime, release qualification and production remain NOT VERIFIED.

## Authority and provenance

- Central source: KaspaPulse/security-governance at 967ed5068947a39961d5d5cc483ef65d25a61059.
- Runtime SHA256: 38309d2ab8fa30096d99940f855e88173faa182e60db33f2a96b2d3408507430.
- Runtime sequence and rollback floor: 2.
- Trust evidence and adapter starting point: KaspaPulse/gov-forms at
  03ed150c4af0fb8b1cd353720e8c9972db4551b6, .security/ksss/.
- Maki and Gov Forms establish the consumer pattern; KGW keeps its own Tauri,
  node, bridge and Windows process boundaries.
- runtime_loader.py adds Windows environment support and a rollback floor.
  trust_acceptance.py changes only the repository label.
- reference-parity.json retains its original Gov Forms schema/provenance.
  Its 34 source comparisons are reused only while their hashes match the
  identical signed runtime. The KGW gate checks that binding on every run.
- Signed trust/evidence bytes are binary-preserved by .gitattributes.
  The policy resolver, classifier, schemas and knowledge engine run directly
  from the verified archive; there is no copied central policy engine.

## Local commands

Use Python 3.12 or newer from the repository root:

```text
python -B tools/kgw_ksss_gate.py
python -B .security/ksss/consumer.py evaluate --base <exact-base-sha> --output <outside-repository.json>
python -B .security/ksss/consumer.py knowledge --context <query.json>
```

Normal evaluation is offline and needs no external KSSS checkout.
For adoption/update, verify release and runtime signatures using pinned Cosign
v3.0.6 and the pinned trusted roots before accepting changed trust material:

```text
python -B .security/ksss/trust_acceptance.py verify --explicit-time <UTC-time> --require-cryptographic --cosign <cosign-path> --output <trust-result.json>
```

CI runs cryptographic acceptance, the offline gate and an exact-base change
evaluation before the existing quality job. No CI workflow was run remotely
during this local-only adoption task. Existing Rust/npm checks are preserved.

## Evidence and learning

The medium profile cannot lower the risk/applicability floor. Eighteen controls
remain required; their operational evidence is NOT VERIFIED in control-state.json.
L1-002 is central-plane-only; L6-001 is conditional, never blocking on AI suspicion alone.
evaluate unions official per-path classifications and reports invalidated controls.
Its identity includes HEAD, content tree and staged/unstaged/untracked/submodule state.
Store outputs outside the repository to avoid changing the evaluated tree.

knowledge accepts context, active_change_types, and a failure_fingerprint or
operation_class. Pass the actual evaluated change types; do not omit known
invalidators to reuse stale evidence. Lookup covers local and bundled global records.
Three existing failures and one historically successful path are indexed.
last_verified_at on failure records is the record/schema review date, not a new
runtime reproduction. The successful path retains its historical execution date.
SERVICE_STATUS and LEARNING_STATUS remain separate; restoration does not close
learning without durable protection. Urgent production containment takes priority.

## Runtime qualification boundary

runtime-contract.json requires six mainnet/testnet10 node and bridge cases.
Each includes actual IPC, settings, owned process/endpoints, raw output, UI state,
stop/restart, crash and close/relaunch proof. Testnet12 stays explicitly opt-in.

```text
python -B .security/ksss/consumer.py release-check --receipt <receipt.json> --artifact <candidate.exe> --evidence-root <directory> --environment-fingerprint <expected-64-hex>
```

This validates a supplied receipt and evidence-file hashes; it does not execute
the application or establish the truth of a test operator's observations.
A reviewer must independently confirm the real packaged application evidence.
Synthetic unit fixtures are never runtime qualification. This command is not a
publisher or deployment workflow. Current task authority ends at a local commit
and local mirror checkpoint; GitHub push, app launch and deployment are excluded.
