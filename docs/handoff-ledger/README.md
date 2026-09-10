# Durable Handoff Ledger

This directory stores concise, durable checkpoints for meaningful Kaspa Gateway engineering transitions. It is the repository-native replacement for project-external chat/session memory.

## Required Checkpoint Fields
- Timestamp and task/status.
- `LAST CONFIRMED STATE`.
- Completed and verified work.
- Commands/actions and relevant changed files.
- Test/evidence results, including failures.
- Root cause/decision when material.
- Remaining work/blockers.
- `NEXT ACTION`.
- `DO NOT REPEAT`.

## Naming
Use `YYYY-MM-DD-<short-task-name>.md`. Update the current task checkpoint during the task; create a new record for a distinct task or incident.

Keep records concise. Git owns code history; CI owns check results; Releases own release history. The ledger preserves resumability and rationale, not duplicate history.
