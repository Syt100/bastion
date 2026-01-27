# Agents

This document describes agent enrollment and day-to-day agent management in the Web UI.

## Enrollment

High-level flow:

1. In the Web UI: **Agents** → create an **enrollment token**
2. On the target machine: run `bastion agent ... --enroll-token <token>`

Enrollment tokens are time-limited (default: 1 hour) and can optionally be configured with a limited number of uses. Treat them as secrets.

Example:

```bash
./bastion agent \
  --hub-url http://127.0.0.1:9876 \
  --enroll-token <token> \
  --name "<friendly-name>"
```

Notes:

- The agent keeps its enrollment identity in its own data directory (`--data-dir` / `BASTION_DATA_DIR`).
- If an agent is already enrolled, `--enroll-token` is not required.

## Agent lifecycle and status

Agents can be:

- **online**: connected recently (Hub shows it as online)
- **offline**: not currently connected (some actions will be queued until it reconnects)
- **revoked**: explicitly revoked; it should no longer be trusted

In the Agents page you can also open an agent detail view that shows config snapshot status and recent errors.

## Labels (grouping and targeting)

Agents can be tagged with free-form labels (e.g., `prod`, `cn`, `db`).

Where labels are used:

- **Agents list filter**: filter agents by labels (AND/OR mode).
- **Bulk operations selector**: target a set of agents by label selection.

Common patterns:

- Environment: `prod`, `staging`, `dev`
- Region: `cn`, `us`, `eu`
- Role: `db`, `web`, `media`

## Config sync (status + actions)

The Hub generates a per-agent “config snapshot” (jobs + secrets + runtime-relevant settings for that agent).
Agents pull/apply this snapshot when online.

In the Web UI (Agents page), each agent exposes:

- **Desired snapshot ID**: what the Hub wants the agent to apply next
- **Applied snapshot ID**: what the agent last reported as applied
- **Last error**: last sync error kind/message and timestamp (if any)

Actions:

- **Sync now** (per agent): attempt to send/prompt the agent to sync immediately
- **Sync config now** (bulk): schedule a bulk operation to prompt multiple agents

Notes:

- If an agent is **offline**, sync requests are recorded and will be delivered when it reconnects.
- For bulk-sync and other bulk actions, track progress in **Settings → Bulk operations**.

## Security actions (rotate key / revoke)

### Rotate agent key

Rotating an agent key generates a new credential for the same agent ID.

- The UI will show the new key once; you need to update the agent’s `agent.json` (in its data dir) and restart the agent.

### Revoke agent

Revoking an agent marks it as revoked on the Hub. A revoked agent should be treated as compromised/untrusted.

If you intend to re-add the machine, enroll it again as a new agent.
