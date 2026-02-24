# Change: Update security and queue stability optimizations

## Why
A follow-up optimization pass found additional high-priority risks: an open Dependabot runtime advisory (`glib`), unbounded offline scheduling/writer queues on the Agent side, and unstable OFFSET pagination in notifications queue listing under concurrent writes.

## What Changes
1. Remediate Dependabot alert `#7` by removing the vulnerable `glib` dependency path from the Windows tray implementation dependency graph.
2. Replace unbounded offline scheduler and offline writer channels with bounded channels plus resilient overflow/closure handling.
3. Add keyset cursor pagination support to notifications queue listing while keeping existing page-based compatibility.
4. Add index optimizations for keyset scan paths used by snapshot and notifications list queries.
5. Harden UI locale switching to be last-write-wins under rapid toggles, and skip dashboard desktop-table prefetch on non-desktop viewports.

## Impact
- Affected specs: `backend`, `ui`
- Affected code:
  - `crates/bastion/Cargo.toml`
  - `Cargo.lock`
  - `crates/bastion/src/agent_client/offline/scheduler/*`
  - `crates/bastion/src/agent_client/offline/storage/writer.rs`
  - `crates/bastion-http/src/http/notifications/queue.rs`
  - `crates/bastion-storage/src/notifications_repo/*`
  - `crates/bastion-storage/migrations/*`
  - `ui/src/stores/ui.ts`
  - `ui/src/views/DashboardView.vue`

## Non-Goals
- No removal of existing page/page_size contracts in current list UIs.
- No broad migration of every list endpoint to cursor-only APIs in this change.
