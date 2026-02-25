# Runs

A **run** is one execution of a job. Runs capture status, timings, progress, and event logs.

## Where to find runs

In the Web UI:

- **Jobs** → pick a job → **Runs**
- Click a run to open the **run detail** page

## Status meanings

- **queued**: accepted and waiting for execution
- **running**: currently executing on the job’s node
- **canceling**: cancel was requested while still running; Bastion is waiting for a safe checkpoint
- **canceled**: finished by user cancellation (terminal)
- **success**: finished successfully (may produce a snapshot)
- **failed**: finished with an error
- **rejected**: rejected due to the job’s overlap policy (e.g., overlap policy is `reject` and a run was already running)

## Cancel a run

In run detail, click **Cancel** when a run is `queued` or `running`.

- If the run is `queued`, it transitions directly to terminal `canceled` and will not start.
- If the run is `running`, Bastion records cancel intent first (`canceling`), then stops at cooperative checkpoints and transitions to terminal `canceled`.
- Cancel requests are idempotent: repeated clicks do not create conflicting state.
- During cancellation, late success/failure writes are ignored once the run is terminal `canceled`.

Notes:

- Cancel is graceful, not process-kill. Some in-flight work may finish before the next checkpoint.
- Existing artifacts already uploaded before cancellation are not automatically deleted.

## Run detail (what you can do)

The run detail page includes:

- **Summary**: status, timings, basic metrics, and the selected source/target
- **Live events**: incremental progress/events (WebSocket) while the run is executing
- **Operations**: restore/verify operations started from this run

For successful runs, you can also start:

- **Restore** (restore files to a destination)
- **Verify** (restores into a temporary directory and verifies file hashes)

See: [Restore and verify](/user/restore-verify).

## Retention note (run history)

The Hub prunes old run history automatically based on **Run retention days**.

Important behavior:

- Runs are **snapshot-aware**: if a successful run still has a snapshot in a “live” status (present/deleting/error), the run record is kept.
- If you delete a snapshot (or it is fully deleted by retention), the corresponding run record can later be pruned when it becomes older than the retention cutoff.

See: [Runtime config](/user/operations/runtime-config).
