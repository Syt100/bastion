# Change: Add Multi-Node Node Context (Per-Node UX Like Single-Node)

## Why
When managing multiple nodes (Hub + Agents), users want each node to feel like a standalone single-node Bastion:
- fewer cross-node distractions (no mixing jobs/targets from different nodes),
- faster navigation (“I’m operating on node X now”),
- deep links that keep the chosen node context across refreshes and sharing.

Today, most screens are effectively global and require manually selecting nodes in specific dialogs.

## What Changes
- Introduce a first-class **node context** in the Web UI.
- Add a node switcher that lets users select:
  - the Hub node, and
  - any enrolled Agent node.
- Add node-scoped routes under `/n/:nodeId/**` so the chosen node persists in the URL.
- In node context, pages like Jobs/Runs/Restore/Verify behave like the single-node app:
  - lists are filtered to the selected node,
  - create/edit defaults to the selected node,
  - cross-node selection is hidden or disabled in node context.
- Keep global management pages (e.g. Agents, global notifications) outside node context.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` router/layout and list/editor pages

