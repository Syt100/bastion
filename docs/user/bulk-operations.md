# Bulk operations

Bulk operations are persistent, async actions applied to a set of agents (nodes).

They are useful for fleet-wide tasks such as updating labels, distributing credentials, syncing config, or cloning jobs.

## Concepts

- **Bulk operation**: a top-level request (kind + selector + payload).
- **Bulk operation item**: a per-agent execution record.

Each item tracks:

- status (`queued`, `running`, `success`, `failed`, `canceled`)
- attempts
- last error kind/message (or a success note)

## UI

- View and manage operations: **Settings → Bulk operations**
  - Open an operation to see per-agent items
  - **Retry failed**: re-queues only failed items
  - **Cancel**: cancels queued items (running items keep running)

Many bulk operations support a **preview** step in the UI to show a dry-run plan.

## Supported kinds (current)

### `agent_labels_add` / `agent_labels_remove`

Add or remove labels on selected agents.

Entry points:

- **Agents** → **Bulk labels**

### `sync_config_now`

Request agents to sync/apply the latest config snapshot.

Entry points:

- **Agents** → **Sync config now**

### `webdav_secret_distribute`

Copy a WebDAV credential from the Hub to selected agents.

Notes:

- Secrets are node-scoped. A job that references a WebDAV secret name requires that secret to exist on the target node.
- This operation can **overwrite** or **skip** if the secret already exists on an agent.

Entry points:

- **Settings → Storage** (Hub node) → **Distribute**

### `job_deploy`

Clone an existing job to selected agents.

Notes:

- Uses a naming template (default `{name} ({node})`) and auto-suffixes on collisions.
- Performs per-node validation (e.g., missing node-scoped secrets).

Entry points:

- **Jobs** → pick a job → **Deploy to nodes**

## API (optional reference)

The Hub exposes:

- `POST /api/bulk-operations` — create an operation
- `POST /api/bulk-operations/preview` — preview (supported kinds only)
- `GET /api/bulk-operations` — list recent operations
- `GET /api/bulk-operations/{id}` — operation detail with items
- `POST /api/bulk-operations/{id}/retry-failed` — retry failed items
- `POST /api/bulk-operations/{id}/cancel` — cancel an operation
