# Change: Add graceful cancellation for running runs and operations

## Why
Backup, restore, and verify tasks can run for a long time, but the system currently does not provide a first-class cancel path for in-flight tasks. Operators need a safe way to stop work without corrupting artifacts, leaving locks open, or causing state races.

## What Changes
- Add a terminal `canceled` status to run and operation lifecycles, plus cancel-request metadata for auditability.
- Add authenticated, idempotent cancel APIs for runs and operations.
- Add in-memory cancellation signaling for active tasks and persist cancel intent so cancellation survives transient restarts.
- Add cooperative cancellation checkpoints in long-running backup/restore/verify execution paths to support graceful interruption and cleanup.
- Extend Hub↔Agent protocol to deliver cancel requests for agent-executed run/operation tasks.
- Harden final-state writes with compare-and-set semantics so late success/failure results cannot overwrite canceled state.
- Update Web UI actions and status rendering for `queued`/`running`/`canceling`/`canceled` behavior.

## Impact
- Affected specs: `backend`, `hub-agent-protocol`, `web-ui`
- Affected code:
  - Storage state/enums/repositories (`crates/bastion-storage/*`)
  - HTTP APIs and websocket handling (`crates/bastion-http/*`)
  - Scheduler worker execution and cancellation registry (`crates/bastion-engine/*`)
  - Backup/restore cooperative interruption (`crates/bastion-backup/*`, `crates/bastion-targets/*`)
  - Agent protocol and task handlers (`crates/bastion-core/*`, `crates/bastion/src/agent_client/*`)
  - UI stores/components for run/operation actions (`ui/src/stores/*`, related views)
  - User documentation (`docs/user/*`)

## Non-Goals
- Force-killing worker processes/threads in the first delivery.
- Introducing per-file rollback semantics beyond existing artifact integrity checks.
- Changing run/operation retention policies.
