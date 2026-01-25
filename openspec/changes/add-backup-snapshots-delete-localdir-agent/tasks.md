## 1. Spec & Design
- [ ] Define the protocol payload shape for snapshot deletion tasks and results
- [ ] Define Agent-side safety checks (avoid deleting arbitrary directories)
- [ ] Validate this change with `openspec validate add-backup-snapshots-delete-localdir-agent --strict`

## 2. Protocol (Hub/Agent)
- [ ] Extend `bastion_core::agent_protocol`:
  - Hub → Agent: snapshot delete task message
  - Agent → Hub: snapshot delete event + snapshot delete result
- [ ] Update Hub WebSocket handler to process the new messages and persist events/state
- [ ] Add protocol unit tests (serde round-trip) for the new message types

## 3. Agent Implementation
- [ ] Add Agent handler for snapshot delete tasks:
  - derive the delete path from the task payload
  - perform safety checks before deletion
  - delete recursively; treat "not found" as success
  - send events + final result back to Hub
- [ ] Add Agent tests for safety checks and idempotency behavior

## 4. Hub Delete Worker: Agent Dispatch
- [ ] When processing `artifact_delete_tasks` for `node_id != HUB` and `target_type=local_dir`:
  - attempt to send the delete task to the Agent
  - if offline/unreachable, transition to retrying with backoff
  - on Agent result, mark the delete task done/failed accordingly
- [ ] Ensure the worker can retry safely (idempotent deletes)

## 5. UI Integration
- [ ] On the snapshots page, surface "waiting for agent" states clearly (status + last error)
- [ ] Allow "retry now" to re-dispatch without resetting audit history

