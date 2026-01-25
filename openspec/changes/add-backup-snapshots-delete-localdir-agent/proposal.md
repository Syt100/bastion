# Change: Agent-Executed Snapshot Deletion for Local Directory Targets

## Why
For `local_dir` targets, snapshot data lives on the execution node's filesystem.

To manage snapshots in a multi-node deployment, the Hub must be able to:
- enqueue deletion for snapshots stored on an Agent
- dispatch deletion work to the correct Agent
- observe progress/errors and allow retries when the Agent is offline

High-level design reference: `docs/backup-snapshots.md`.

## What Changes
- Extend the Hub/Agent protocol with snapshot deletion messages:
  - Hub → Agent: delete task request (local_dir)
  - Agent → Hub: delete events + final result
- Implement Agent-side execution:
  - delete `<base_dir>/<job_id>/<run_id>/` with safety checks
  - best-effort event emission (start/progress/complete/error)
- Implement Hub-side dispatch + reconciliation:
  - process `artifact_delete_tasks` whose `node_id` is an Agent and `target_type=local_dir`
  - deliver tasks when Agent is connected; retry with backoff when offline
  - update `artifact_delete_tasks` + `artifact_delete_events` based on Agent results

## Scope
- `local_dir` snapshot deletion where the snapshot is stored on an Agent node.
- Ensures offline tolerance (eventual execution when the Agent reconnects).

## Out of Scope (Follow-ups)
- Agent-executed deletion for other target types (e.g., WebDAV, S3).
- Advanced "lease" semantics and at-least-once delivery hardening beyond idempotency.

## Key Decisions
- **Executor selection**: delete is executed by the node that owns the snapshot (`run_artifacts.node_id`).
- **Idempotency**: Agent deletion treats missing directories as success.
- **Safety**: Agent MUST validate the delete path looks like Bastion run data before removing.

## Risks
- Offline agents can delay deletion indefinitely; the UI must make this visible and actionable.
- Protocol changes require careful versioning and backward compatibility (but project is pre-release).

## Success Criteria
- Snapshots stored on Agents can be deleted from the Hub UI.
- If an Agent is offline, delete tasks remain queued/retrying and succeed once the Agent reconnects.
- Operators can inspect events/errors for Agent-executed deletions.

