# Runtime config (Hub settings)

Bastion exposes a small set of Hub runtime configuration in the Web UI.

In the Web UI:

- **Settings → Runtime config**

## Effective vs saved values

The runtime config page shows:

- **Effective value**: what the Hub is currently using
- **Saved value**: what is stored in the database (used on the next restart when not overridden)
- **Source**: where the effective value comes from (`cli`, `env`, `db`, `default`)

Precedence:

1. CLI flags
2. Environment variables
3. Saved (database) value
4. Built-in default

If a field is overridden by CLI/env, saving a different value in the UI will not change the effective value until you remove the override and restart.

## Settings

### Hub timezone

Default schedule timezone for new jobs (IANA timezone name):

- Examples: `UTC`, `Asia/Shanghai`, `America/Los_Angeles`
- Env: `BASTION_HUB_TIMEZONE`

This does not change existing jobs that already have an explicit schedule timezone.

### Run retention days

How long to keep run history in the database (default: 180):

- Env: `BASTION_RUN_RETENTION_DAYS`

Notes:

- Run pruning is **snapshot-aware**: a run is kept as long as it still has a “live” snapshot (present/deleting/error).

### Incomplete cleanup days

Whether to automatically clean up old, incomplete/failed runs (default: 7):

- Env: `BASTION_INCOMPLETE_CLEANUP_DAYS`
- `0` disables the incomplete cleanup loop

See: [Maintenance (incomplete cleanup)](/user/operations/maintenance).

### Logging

Logging can be configured via the runtime config page:

- Log filter: `BASTION_LOG` / `RUST_LOG`
- Log file: `BASTION_LOG_FILE`
- Rotation: `BASTION_LOG_ROTATION` (`daily|hourly|never`)
- Keep files: `BASTION_LOG_KEEP_FILES`

See: [Logging](/user/operations/logging).

### Default backup retention (new jobs)

These defaults are applied when you create a **new** job in the job editor:

- enabled
- keep last / keep days
- max delete per tick / per day (safety limits)

Notes:

- Changing these defaults does not change existing jobs.
- This setting is used by the UI immediately (no restart needed).

## Restart note

Most runtime config fields are loaded at Hub startup.

After changing Hub timezone / run retention / incomplete cleanup / logging settings, restart the Hub to apply the new values.

