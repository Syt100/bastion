# Change: Add Agent Labels (Tags) + Label Filtering

## Why
Multi-node management becomes hard once there are dozens of agents: operators need a way to group nodes (e.g., `prod`, `shanghai`, `db-backup`) and target those groups consistently.
Today, agents can only be addressed by id/name and the UI cannot filter or select nodes by a reusable grouping primitive.

Agent labels are the foundation for:
- Label-based filtering in the UI.
- Label-based selectors for future bulk operations (separate change).

## What Changes
- Storage:
  - Persist agent labels in the DB (many-to-many: agent → labels).
- Backend:
  - Extend agent list/detail APIs to include labels.
  - Add APIs to manage labels on an agent (add/remove/set).
  - Add API to list all labels with usage counts for UI autocomplete.
  - Support label filtering in agent list: `labels[]` with `labels_mode=and|or` (default `and`).
- Web UI:
  - Render agent labels as tags in the Agents page.
  - Add a label filter (multi-select) with AND/OR toggle.
  - Add an agent label editor.

## Non-goals (This Change)
- Generic bulk operations framework (covered by `add-bulk-operations-framework`).
- Job/secrets distribution flows (covered by later bulk changes).

## Decisions
- Label filter default is **AND**; UI MUST allow switching to **OR**.
- Label format is constrained (lowercase, limited charset) to avoid “almost duplicate” labels.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (expected): `crates/bastion-storage` (migrations/repo), `crates/bastion-http` (agents APIs), `ui` (Agents page + store)

