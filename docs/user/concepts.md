# Concepts and terminology

This page defines the core concepts used throughout Bastion and the Web UI.

## Hub, Agent, and nodes

- **Hub**: the main Bastion service (HTTP API + Web UI + scheduler). In the UI and API, the Hub is the special node id `hub`.
- **Agent** (optional): a remote worker that connects to the Hub over WebSocket and executes jobs on another machine.
- **Node**: a generic term meaning either the Hub or an Agent. A job always runs on exactly one node.

## Jobs, runs, and snapshots

- **Job**: the configuration of a backup (source + target + schedule + retention + notifications).
- **Run**: one execution of a job (queued → running → success/failed/rejected). Runs have logs/events and may produce backup output.
- **Snapshot**: the stored backup output produced by a successful run (plus an index record in the Hub).
  - Snapshots are what you pin/delete/retain.
  - Snapshots are separate from runs so you can keep backup data even if old run history is pruned.

## Operations (restore / verify)

An **operation** is a long-running action started from a successful run:

- **Restore**: restore some or all files from a snapshot into a local directory or a WebDAV destination.
- **Verify**: a “restore drill” that restores into a temporary directory and verifies file hashes (and SQLite integrity when applicable).

Operations have their own progress and event logs in the run detail view.

## Secrets (credentials)

Bastion stores credentials as **encrypted secrets** in the Hub database.

- **Node-scoped** secrets (important): many secrets exist separately per node (Hub vs each Agent).
  - Example: WebDAV credentials must exist on the node that will upload/download WebDAV data.
  - Use bulk operations to distribute Hub secrets to agents when needed.

## Bulk operations

A **bulk operation** is an async action applied to many agents (e.g., add/remove labels, sync config now, distribute WebDAV credentials, deploy a job).

- Each bulk operation contains per-agent **items** with their own status and error information.

