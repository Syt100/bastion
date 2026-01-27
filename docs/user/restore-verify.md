# Restore and verify

Restore and verify are long-running **operations** started from a **successful run**.

## Start an operation

In the Web UI:

1. Open a job’s **Runs**, then open a **successful** run.
2. Click **Restore** or **Verify**.

Operations are tracked in the run detail view (status + progress + event log).

## Restore

Restore reads a snapshot and writes the restored files to a destination.

### Destinations

#### Local filesystem

- **Node**: where the restore runs (Hub or an Agent)
- **Destination directory**: a directory on that node

Notes:

- The directory path is interpreted on the selected node’s filesystem.
- The **Browse** button requires the node to be online.

#### WebDAV

- **Base URL**: e.g. `https://dav.example.com/backup-restore`
- **Secret name**: selects stored WebDAV credentials
- **Prefix**: destination folder under the base URL

Important:

- WebDAV credentials are **node-scoped**. The restore will only work if the WebDAV secret exists on the node executing the restore.

See: [Storage (WebDAV)](/user/storage).

### Conflict policy

When a restored path already exists at the destination:

- **overwrite**: overwrite existing files
- **skip**: keep existing files and skip conflicts
- **fail**: stop the restore with an error on the first conflict

### Selection (optional)

You can restore:

- everything (default), or
- only selected files/directories from the run entries list

## Verify (restore drill)

Verify is a safety check:

1. Fetches the snapshot
2. Restores it into a **temporary directory**
3. Verifies file hashes against the snapshot index
4. Runs SQLite integrity checks when applicable

If verification fails, the operation is marked as **failed** and the event log includes sample errors.

## Multi-node notes and current limitations

- **Encrypted backups + agent restore**: if you restore an encrypted backup onto an Agent, the Hub automatically ensures the agent has the required private key before dispatching the restore.
- **Verify runs on the Hub** (current behavior). That means:
  - WebDAV snapshots are verifiable as long as the Hub has the WebDAV secret.
  - Local directory snapshots produced on an Agent are typically **not** verifiable from the Hub unless the snapshot directory is accessible to the Hub (e.g., a shared mount).

If you care about Hub-side verify drills in a multi-node setup, prefer using **WebDAV** as the target.

