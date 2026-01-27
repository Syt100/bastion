# Architecture

## High-level components

- **Hub** (single binary): HTTP API + Web UI, scheduler/queues, SQLite metadata storage, encrypted secrets.
- **Agent** (Hub subcommand): connects to the Hub over WebSocket and executes jobs on a remote machine.
- **Web UI** (`ui/`): Vue 3 + Vite single-page app.

## Data model (conceptual)

- **Jobs**: what to run, where to run, and when to run (schedule + timezone + overlap policy).
- **Runs**: execution records (status/progress/summary/events).
- **Run artifacts (snapshots)**: index records for successful runs that produced stored backup output.
- **Secrets**: encrypted at rest in the Hub database; many are **node-scoped** (Hub vs each Agent).

## Background workers (examples)

- **Artifact delete queue**: asynchronous snapshot deletion with retries and an event log.
- **Snapshot retention loop**: server-enforced retention based on job policies (keep last / keep days), respecting safety limits.
- **Incomplete cleanup**: cleanup of incomplete/failed staging directories.
- **Run retention**: prunes old run history while keeping runs that still have “live” snapshots.
- **Notifications loop**: sends queued WeCom/email notifications when runs finish.

For user-facing behavior and UI entry points, see the [User manual](/user/).
