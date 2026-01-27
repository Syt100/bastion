# Agents

This document describes agent enrollment and day-to-day agent management in the Web UI.

## Enrollment

High-level flow:

1. In the Web UI: **Agents** → create an **enrollment token**
2. On the target machine: run `bastion agent ... --enroll-token <token>`

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
