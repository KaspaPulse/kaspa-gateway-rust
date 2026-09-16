# Contributing to Kaspa Gateway

Thank you for helping improve Kaspa Gateway. This repository accepts changes through GitHub pull requests to `main`.

## Before opening a change

- Use GitHub Issues for reproducible bugs, enhancement proposals, and questions that need project discussion.
- Report suspected vulnerabilities privately using the process in [`SECURITY.md`](SECURITY.md); do not open a public issue for an active vulnerability.
- Keep each change narrowly scoped. Avoid unrelated dependency, formatting, generated-file, or architecture churn.
- Never commit credentials, private keys, seed phrases, tokens, or other secrets.

## Contribution process

1. Create a branch from the current `main`.
2. Make the smallest change that addresses the issue or proposal.
3. Add or update automated tests when behavior changes.
4. Run the checks relevant to the files you changed.
5. Open a pull request to `main` describing the change, risks, and verification performed.
6. Address review and required-check failures without weakening the checks.
7. The protected repository flow uses reviewed pull requests and squash integration; do not rely on direct pushes to `main`.

## Coding and documentation standards

### Rust

- Format with `cargo fmt`.
- Keep Clippy clean under the repository's strict `-D warnings` policy.
- Preserve typed interfaces and explicit runtime/process ownership contracts; do not bypass existing owners with duplicate control paths.
- Use locked dependencies for qualification and keep lockfile changes limited to the intended dependency update.

### JavaScript / Tauri frontend

- Follow the repository ESLint configuration; do not add blanket rule suppressions to make a gate pass.
- Keep user-visible state derived from backend/runtime truth rather than optimistic UI-only state.
- Keep accessibility/status semantics intact when changing interactive surfaces.

### Documentation

- Update user, security, architecture, API, or operational documentation when a change makes existing statements inaccurate.
- Keep release/version claims tied to published GitHub release metadata rather than an unpublished candidate.

## Testing policy

Major new functionality must include automated tests that exercise the new behavior. Important bug fixes should include a regression test that fails before the fix and passes after it whenever practical.

Start with focused tests for the affected path, then run broader gates only when the changed inputs require them. Common repository checks include:

```bash
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
```

For the desktop frontend:

```bash
cd apps/kaspa-gateway-desktop
npm ci
npm run lint
```

The pull request must pass the repository's required CI, security, dependency, secret-scanning, workflow-lint, and analysis checks before integration.

## Pull request description

State what changed, why it changed, which behavior or contract is affected, and the exact tests or checks used to verify it. Call out security, dependency, release, migration, compatibility, or runtime implications explicitly when applicable.
