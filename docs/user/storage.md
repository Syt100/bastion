# Storage (WebDAV)

Bastion currently supports WebDAV as a remote storage target.

WebDAV credentials are stored as encrypted secrets in the Hub’s database and are **node-scoped**:

- Hub secrets live under the reserved node id `hub`
- Each agent has its own secret namespace

This means:

- A job that runs on an agent and references a WebDAV credential name requires that credential to exist on that agent.
- Having the credential on the Hub alone is not sufficient for agent-run jobs.

## Manage WebDAV credentials

In the Web UI:

- **Settings → Storage** (Hub context by default)

Storage settings are shown in a node context (`hub` or a specific agent). You can:

- create/edit/delete credentials by name
- view update timestamps
- copy credential names for use in jobs

## Distribute a WebDAV credential to agents

To copy a Hub credential to many agents:

1. Ensure the credential exists on the **Hub** (node `hub`)
2. Click **Distribute**
3. Select target agents (by labels or explicit node IDs)
4. (Optional) set **overwrite**
5. Preview, then create the bulk operation

Progress and per-agent results are tracked in:

- **Settings → Bulk operations**

## Where WebDAV credentials are used

- **Job targets**: when you pick target type **WebDAV**, you select a credential name.
- **Restore to WebDAV**: restore uses the executor node’s WebDAV secret scope, so agent-run restores may require distributing the same credential to that agent first.

See: [Restore and verify](/user/restore-verify).
