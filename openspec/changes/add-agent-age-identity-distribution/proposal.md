# Change: Allow distributing backup age identities to selected Agents

## Why
To support executing restores on Agents (especially when restoring to an Agent’s local filesystem), encrypted backups must be decryptable on the executor node.

Today, backup age identities (`backup_age_identity`) are stored on the Hub only. This prevents Agents from restoring encrypted runs unless the Hub performs decryption and acts as a data transform node.

## What Changes
- Support distributing a backup age identity (`backup_age_identity/<key_name>`) from the Hub to a selected Agent on demand.
- Store the distributed identity in the existing encrypted secrets store under the Agent node scope.
- Extend the Hub→Agent secrets snapshot to include the distributed age identity secrets so Agents can persist them locally (encrypted-at-rest on the Agent).
- Add audit-friendly operation events/log entries for distribution and usage (without leaking secret values).

## Impact
- Affected specs: `backend`, `hub-agent`, `hub-agent-protocol`
- Affected code:
  - Secrets store: `crates/bastion-storage/src/secrets_repo/*`
  - Agent snapshots: `crates/bastion-engine/src/agent_manager/*`, `crates/bastion/src/agent_client/managed/*`
  - Restore orchestration: restore dispatcher (Agent-executed restore)

## Compatibility / Non-Goals
- No new UI for key management in this change; distribution is driven by restore execution needs.
- This change does not alter backup encryption semantics.

