# Storage (WebDAV credentials)

Bastion currently supports WebDAV as a remote storage target.

WebDAV credentials are stored as encrypted secrets in the Hub’s database and are **node-scoped**:

- Hub secrets live under the reserved node id `hub`
- Each agent has its own secret namespace

This means:

- A job that runs on an agent and references a WebDAV credential name requires that credential to exist on that agent.
- Having the credential on the Hub alone is not sufficient for agent-run jobs.

## Manage WebDAV credentials

In the Web UI:

- **Settings → Storage → WebDAV credentials**

You can create/edit/delete credential entries by name.

## Distribute a WebDAV credential to agents

To copy a Hub credential to many agents:

1. Ensure the credential exists on the **Hub** (node `hub`)
2. Click **Distribute**
3. Select target agents (by labels or explicit node IDs)
4. (Optional) set **overwrite**
5. Preview, then create the bulk operation

Progress and per-agent results are tracked in:

- **Settings → Bulk operations**

