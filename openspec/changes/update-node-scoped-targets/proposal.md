# Change: Update Node-Scoped Targets & Credentials (Hub-Managed, Per-Node Isolation)

## Why
For a multi-node system, storage targets and their credentials behave like local configuration:
- A WebDAV credential configured for node A should not implicitly apply to node B.
- Jobs for a node should only be able to reference targets/credentials available on that node.
- This isolation reduces accidental cross-node backups and makes the node context feel like the single-node app.

We also need to support Agent offline execution in a later milestone, which requires Agents to have local copies of their targets/credentials.

## What Changes
- Make backup targets and credentials **node-scoped**:
  - Targets (WebDAV, local-dir) belong to exactly one node.
  - Credentials referenced by targets belong to the same node.
- Web UI:
  - Storage/Targets pages operate in node context and show only targets for that node.
  - Job creation/edit in node context can only choose targets for that node.
- Hub/Agent:
  - The Hub remains the control plane and stores node-scoped configuration.
  - The Hub can sync node-scoped targets/credentials to Agents (used by later offline execution work).
- Managed Agent local UI (if enabled in the future) SHALL be read-only for configuration and MUST direct users to the Hub for edits.

## Impact
- Affected specs: `backup-jobs`, `hub-agent`, `web-ui`, `targets-webdav`, `targets-local-dir`
- Affected code: backend storage schema + APIs, agent protocol, and Web UI configuration pages

