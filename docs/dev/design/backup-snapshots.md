# Backup snapshot management (Snapshots / Artifacts)

This document describes Bastion's snapshot (artifact) management subsystem: snapshot indexing, async deletion, and retention enforcement.

For user-facing behavior and UI entry points, start with the user manual: [Backup snapshots](/user/backup-snapshots).

## Goals

- Treat successful backup outputs as a first-class resource (**snapshots**), separate from run history.
- Support manual delete (single/bulk) with retries and observable progress.
- Enforce retention policies server-side (keep last N / keep days), with safety limits.
- Correctly handle multi-node execution (Hub vs Agent): deletion must run on the node that can actually access the data.

## Concepts

- **Run**: one execution of a job. A successful run may produce stored backup output.
- **Snapshot (artifact)**: the stored backup output plus an index record in the Hub database.
- **Delete task**: an async task queue entry that deletes snapshot data from the target.
- **Retention tick**: a server loop that selects snapshots to delete based on job retention policy.

## Data model (SQLite)

The system stores snapshot management state in three main tables:

- `run_artifacts`: snapshot index records (one per successful run that produced a stored output)
- `artifact_delete_tasks`: delete queue (one per run snapshot)
- `artifact_delete_events`: append-only event log for delete tasks

### `run_artifacts` (snapshot index)

Key fields (conceptually):

- `run_id` (primary key), `job_id`, `node_id`
- `target_type`, `target_snapshot_json`
- `artifact_format`
- `status`: `present | deleting | deleted | missing | error`
- `ended_at`, `pinned_at`, `deleted_at`, and optional error/attempt metadata

Status notes:

- `present`: snapshot exists and is available
- `deleting`: a delete task is queued/running
- `deleted`: delete completed successfully
- `missing`: treated as already gone (e.g., deleted outside Bastion)
- `error`: delete failed and requires attention / retries / ignore

### `artifact_delete_tasks` (delete queue)

Delete tasks follow a state machine similar to other background task queues:

- `status`: `queued | running | retrying | blocked | abandoned | done | ignored`
- retry metadata: `attempts`, `next_attempt_at`, `last_error_kind`, `last_error`, ...

### `artifact_delete_events` (event log)

An append-only log:

- `run_id`, `seq`, `ts`, `level`, `kind`, `message`, `fields_json`

This supports UI "delete log" views and post-mortem debugging.

## Runtime flows

### 1) Index creation on successful run

When a run completes with `success`:

1. The system resolves the run's target snapshot data (`target_snapshot_json`).
2. It extracts useful metadata for list views and retention decisions.
3. It upserts a `run_artifacts` record with `status = present`.

### 2) Manual snapshot delete (single/bulk)

When the API receives a delete request:

1. Validate the snapshot exists and is deletable.
   - If snapshot is pinned, require an explicit "force" confirmation.
2. Upsert a delete task (`artifact_delete_tasks`) with `status = queued`.
3. Update the snapshot index to `status = deleting`.
4. Append an event to `artifact_delete_events` (queued).

Delete execution:

- The delete worker claims tasks and executes target-specific deletion.
- Deletion is idempotent: "not found" on the target becomes `missing` / treated as success.

### 3) Retention enforcement

The retention loop runs periodically on the Hub:

1. Load a job's retention policy (keep last / keep days + safety limits).
2. List snapshots that are `present` and not pinned.
3. Compute a keep-set and delete-set.
4. Enqueue delete tasks for the delete-set, respecting safety limits (per tick / per day).

Pinned snapshots are excluded from retention deletes.

### 4) Multi-node execution rules

Deletion must run where the data and credentials are available:

- `local_dir` target: delete must run on the node that produced/stores the data (Hub or the specific Agent).
- `webdav` target: delete requires WebDAV credentials in the executor node scope.

In practice:

- Hub can perform WebDAV deletion if it holds the WebDAV secret.
- Agent-based deletion is required for agent-local filesystem data.

## Interactions with run retention

Run history pruning is snapshot-aware:

- The run retention loop prunes old runs **only if** the corresponding snapshot is not "live".
- Runs with snapshots in `present | deleting | error` are kept, so snapshot management stays possible even when old run history is pruned.

## Pointers (implementation)

- HTTP routes: `crates/bastion-http/src/http/mod.rs`
- Docs server (in-app): `crates/bastion-http/src/http/docs.rs`
- Storage migrations: `crates/bastion-storage/migrations/` (look for snapshot/artifact migrations)
- Retention loops: `crates/bastion-engine/src/scheduler/`

