# Change: Add Bulk Operations Framework (Async, Retryable)

## Why
Batch management (labels, config sync, secrets distribution, job deploy) should not be implemented as “loop over N nodes in a single HTTP request”.
That approach is brittle (timeouts, partial failures), hard to observe, and provides a poor operator experience.

We need a generic bulk operations framework that:
- Accepts a node selector (explicit ids or label selector).
- Persists a bulk operation and per-node items.
- Processes items asynchronously with bounded concurrency.
- Continues on failures and supports retrying failed items.
- Is observable in the UI.

## What Changes
- Storage:
  - Add `bulk_operations` and `bulk_operation_items` tables.
- Backend:
  - Add authenticated APIs to create/list/get/cancel/retry bulk operations.
  - Add a background worker to process queued bulk items with bounded concurrency.
  - Implement the first bulk action: “bulk label update” (add/remove labels) to validate the framework end-to-end.
- Web UI:
  - Add a bulk operations panel/page showing operation progress and per-node results.
  - Add an entry point on the Agents page to start a bulk label update.

## Decisions
- Bulk operations MUST continue on per-node failures; failures are reported per-node.
- Bulk selection supports:
  - explicit `node_ids[]`
  - or `labels[]` with `labels_mode=and|or` (default `and`)
- Bulk secret/job operations will be added as separate changes using this framework.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (expected): `crates/bastion-storage` (migrations/repo), `crates/bastion-engine`/`crates/bastion` (worker loop), `crates/bastion-http` (APIs), `ui` (panel + triggers)

