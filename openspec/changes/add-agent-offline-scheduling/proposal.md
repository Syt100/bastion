# Change: Add Agent Offline Scheduling & Execution (Cache Config, Run Offline, Sync Back)

## Why
In multi-node deployments, Agents may temporarily lose connectivity to the Hub due to network issues or maintenance.
For backups, “Hub unreachable” should not automatically mean “no backups happen”.

To make each node feel like a standalone single-node Bastion:
- Agents SHOULD continue scheduled backups even when the Hub is unreachable.
- Agents MUST cache the last-known job configuration and target credentials locally.
- When connectivity returns, Agents MUST sync run history and results back to the Hub so the Hub UI remains the source of truth.

The Hub remains the control plane for configuration edits (single-user), and enrolled Agents are managed/read-only.

## What Changes
- Add a config sync mechanism from Hub → Agent so Agents persist the last-known:
  - job definitions (including schedules and overlap policy),
  - target definitions and credentials required to execute those jobs.
- Add an Agent-side scheduler that triggers runs based on cached schedules when Hub connectivity is unavailable.
- Persist run history and events locally on the Agent and sync them to the Hub when connected.
- Hub ingests Agent-produced runs/events and displays them in the UI like normal runs.
- Notifications remain Hub-scoped; for offline Agent runs, notifications may be delayed until the Hub receives results.

## Impact
- Affected specs: `hub-agent`, `backup-jobs`, `web-ui`
- Affected code: agent protocol/messages, agent local storage, scheduler, and run ingestion paths

