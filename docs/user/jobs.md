# Jobs

A **job** defines what to back up, where to store it, and when to run.

## Where jobs run (Hub vs Agent)

Each job runs on exactly one node:

- **Hub (local)**: runs on the Hub machine.
- **Agent**: runs on a specific enrolled agent machine.

In the Web UI, the Jobs page is shown in a **node context** (`hub` or a specific agent). Switch nodes using the node selector in the main layout.

## Create and edit jobs

In the Web UI:

- **Jobs** → **Create**

The job editor is split into steps:

- **Basics**: name, node, schedule, timezone, overlap policy, retention
- **Source**: what to back up (depends on job type)
- **Target**: where to store the backup output (local directory or WebDAV)
- **Security**: artifact format + optional encryption
- **Notifications**: inherit or customize destinations
- **Review**: final summary (optional JSON preview)

## Scheduling and overlap

Each job has:

- **Schedule mode**
  - **Manual**: no schedule; you trigger runs via **Run now**
  - **Simple**: UI-friendly presets (stored as cron under the hood)
  - **Cron**: advanced cron schedule string
- **Schedule timezone**: an explicit IANA timezone string used to interpret the schedule (independent of OS timezones)
  - New jobs default to the Hub timezone.
- **Overlap policy**
  - **queue**: triggers while running are queued
  - **reject**: triggers while running are rejected

## Job types (source settings)

### Filesystem

Backs up files/directories on the selected node.

Source settings:

- **Source paths**: one or more paths to include
  - The **Browse** button uses the node’s filesystem. If you picked an Agent, the Agent must be online for browsing.
- **Pre-scan**: estimate totals before packaging (useful for progress/ETA)
- **Include/Exclude**: line-based patterns
- **Symlink policy**: keep / follow / skip
- **Hardlink policy**: copy / keep
- **Error policy**: fail fast / skip fail / skip ok

### SQLite

Creates an online SQLite snapshot (`sqlite backup` API) and then packages it as a backup artifact.

Source settings:

- **SQLite path**: path to the database file on the selected node
- **Integrity check (optional)**: run `PRAGMA integrity_check` on the snapshot and fail the run if it reports problems

### Vaultwarden

Backs up a Vaultwarden deployment (SQLite `db.sqlite3` + required data directory contents).

Source settings:

- **Vaultwarden data dir**: host path of Vaultwarden’s `data/` directory on the selected node

For a concrete Vaultwarden setup, see the recipe: [Vaultwarden](/user/recipes/vaultwarden).

## Targets (where backups are stored)

### Local directory

Stores backup output under a directory on the selected node:

- **Base dir**: e.g. `/opt/bastion-backups`

### WebDAV

Stores backup output to a WebDAV server:

- **Base URL**: e.g. `https://dav.example.com/backups`
- **Secret name**: selects credentials stored in Bastion (WebDAV secrets are **node-scoped**)

Manage credentials in **Settings → Storage** (and distribute them to agents when needed): [Storage (WebDAV)](/user/storage).

### Part size

Targets support a **part size** (MiB). Larger backups are split into multiple parts to avoid huge single files and to make retries cheaper.

## Artifact format and encryption

### Format

- **archive_v1**: a compressed archive format (recommended default)
- **raw_tree_v1**: a “raw file tree” format (no payload encryption support)

Note: Vaultwarden jobs currently support **archive_v1** only.

### Encryption (age)

For `archive_v1`, you can enable payload encryption (age x25519).

- **Encryption key name** is a label (default: `default`)
- The Hub auto-creates the key on first use
- Agents receive only the public recipient for encryption; for restore-to-agent, the Hub will distribute the required private key automatically as part of starting the restore

## Backup snapshots and retention

Successful runs produce a **backup snapshot** (the stored backup output). You can:

- view/pin/delete snapshots per job: [Backup snapshots](/user/backup-snapshots)
- configure retention on the job:
  - keep last / keep days
  - safety limits (max deletes per tick / per day)
  - **defaults** come from **Settings → Runtime config** when you create a new job

## Notifications (per job)

Jobs support per-run notifications (WeCom bot + email).

- **Inherit**: send to all enabled destinations
- **Custom**: select destinations for this job (disabled destinations are ignored)

Configure channels/destinations/templates in **Settings → Notifications**.

See: [Notifications](/user/operations/notifications).

## Deploy (clone) a job to nodes

The UI provides a bulk **Deploy to nodes** action to clone a job onto many agents.

What deploy does:

- creates a new job for each selected agent
- preserves the source job’s spec + schedule + timezone + overlap policy
- validates node-scoped requirements (for example, missing WebDAV secrets)
- triggers a config sync after creating each job (offline agents apply on next connect)

Naming template:

- default: `{name} ({node})`
- placeholders: `{name}`, `{node}`
- collisions are auto-suffixed (`#2`, `#3`, …)

Progress is tracked in **Settings → Bulk operations**.

## Runs, restore, and verify

From a job, you can open:

- **Runs**: history (status, timings, errors)
- **Snapshots**: backup outputs

Successful runs support:

- **Restore**: restore a run (full or selected entries) to a local directory or WebDAV destination
- **Verify**: a “restore drill” to a temporary directory + file hash verification; also runs SQLite integrity checks when applicable

See:

- [Runs](/user/runs)
- [Restore and verify](/user/restore-verify)

## Archive and delete jobs

Jobs can be archived to stop scheduling and hide them from the default view.

- **Archive**: disables “Run now” and other mutating actions; you can optionally “cascade delete snapshots” (pinned snapshots are skipped)
- **Unarchive**: makes the job active again
- **Delete**: permanently removes the job from the Hub database (separate from snapshot deletion)
