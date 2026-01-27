# Backup snapshots

A **backup snapshot** is the output produced by a successful job run (the actual backup data stored in your target, plus a small index record in the Hub).

In the Web UI, snapshots are a first-class resource: you can **pin**, **delete**, and apply **retention policies**.

## Where to find snapshots

In the Web UI:

- **Jobs** → open a job → **Snapshots**

Snapshots are listed per job and include the run time, target, format, size/metrics (when available), and status.

## Status meanings

- **present**: snapshot exists and is available
- **deleting**: a delete task has been queued or is running
- **deleted**: snapshot was deleted successfully
- **missing**: snapshot is not found in the target (treated as “already gone”)
- **error**: deletion failed and may be retrying/blocked/abandoned (check the delete log)

## Pin (protect) a snapshot

Pinning is a safety mechanism:

- Pinned snapshots are excluded from automated retention deletes.
- Manual deletion of pinned snapshots requires an explicit “force” confirmation in the UI.

## Delete snapshots (single / bulk)

Deletion is asynchronous (to avoid HTTP timeouts and to support retries).

Typical flow:

1. Select one or more snapshots.
2. Click **Delete** and confirm.
3. The snapshot moves to **deleting** while the background task runs.
4. Use **Delete log** to see progress, errors, and retry/ignore actions.

If deletion fails:

- **Retry now** re-queues the task immediately.
- **Ignore** marks the task as ignored (useful if you intentionally deleted the data outside Bastion).

## Retention (keep last N / keep days)

Retention is enforced by the server and helps keep storage usage under control.

In the job editor you can configure:

- **keep last**: keep the most recent N snapshots
- **keep days**: keep snapshots newer than N days
- **max delete per tick / per day**: safety limits to avoid mass deletion

The UI supports:

- **Preview**: show what would be deleted
- **Apply now**: enqueue retention deletes immediately

Pinned snapshots are never deleted by retention.

## Multi-node notes (Hub vs Agent)

- **local_dir targets**: snapshot data lives on the node that ran the job (Hub or an Agent). Delete tasks must execute on that node, so offline agents can delay deletion.
- **WebDAV targets**: snapshots live in remote WebDAV storage. Deletion requires credentials on the executing node (often the Hub unless you distribute secrets to agents).

