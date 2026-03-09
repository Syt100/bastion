# Change: Clarify node context navigation (preferred node, node-scoped cues)

## Why
Bastion has a mix of:
- global pages (Dashboard, Agents, global Settings)
- node-scoped pages (`/n/:nodeId/...` such as Jobs, Runs, node-scoped Storage)

Today the node selector can unexpectedly navigate users away from the current page (e.g. selecting a node from a global page jumps to that node’s Jobs).
This creates confusion about “what scope am I operating in?” and increases navigation friction.

## What Changes
- Add a **preferred node** concept used as the default target for node-scoped navigation when the current route is not node-scoped.
- Update the node selector behavior:
  - On node-scoped routes, it switches the node (as today).
  - On non-node-scoped routes, it updates the preferred node without changing the current route.
- Add clearer UI cues on node-scoped pages:
  - The page header shows the active node context (Hub vs Agent name/status) alongside the title.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/layouts/AppShell.vue`
  - `ui/src/router/index.ts` (if routing helpers are needed)
  - `ui/src/stores/ui.ts` (preferred node state)

## Non-Goals
- Making every page node-scoped.
- Any backend API changes.

