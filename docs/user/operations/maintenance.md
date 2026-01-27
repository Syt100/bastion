# Maintenance (incomplete cleanup)

Bastion creates temporary “staging” data during runs. If a run fails, is interrupted, or the Hub/Agent crashes, some temporary data may remain.

To keep disk usage under control, Bastion can automatically clean up old incomplete runs using an **incomplete cleanup** task queue.

In the Web UI:

- **Settings → Maintenance → Cleanup**

## What is being cleaned

Incomplete cleanup tasks target runs that are **not successful** (failed/rejected) and older than the configured cutoff.

Depending on the run’s target type, cleanup may include:

- removing local staging directories, and/or
- cleaning up partial remote outputs (e.g., WebDAV uploads that didn’t finish)

This is separate from snapshot deletion/retention (see [Backup snapshots](/user/backup-snapshots)).

## Status meanings

- **queued**: waiting to run
- **running**: currently executing
- **retrying**: failed previously and will retry later
- **blocked**: cannot make progress automatically (requires user action or environment change)
- **abandoned**: gave up after too many retries / too old
- **done**: cleaned successfully
- **ignored**: explicitly ignored by a user

## What you can do in the UI

- **Retry now**: schedule an immediate retry (use after fixing the underlying issue)
- **Ignore**: stop retrying this task (use if you cleaned up manually or accept the leftover)
- **Unignore**: put an ignored task back into the queue
- Open a task to view the **event log** and the last error details

## Configuration

The cutoff is controlled by **Incomplete cleanup days**:

- In the Web UI: **Settings → Runtime config**
- CLI/env: `--incomplete-cleanup-days` / `BASTION_INCOMPLETE_CLEANUP_DAYS`

Notes:

- Default is `7` days.
- Set to `0` to disable the incomplete cleanup loop (you will need to clean up manually).

