# Defaults and configuration precedence

This page summarizes Bastion's default behavior and how configuration is resolved.

## Default Hub behavior

When you run `bastion` with no flags:

- bind: `127.0.0.1:9876`
- data directory: auto-resolved (see below)
- Web UI: available at `http://127.0.0.1:9876/`

## Data directory resolution

Priority order:

1. `--data-dir <path>` (CLI)
2. `BASTION_DATA_DIR=<path>` (env)
3. `<exe_dir>/data` if writable
4. OS fallback directory:
   - Windows: `%PROGRAMDATA%\\bastion\\data` (if available)
   - Otherwise: OS-specific app local data dir

See: [Data directory](/user/operations/data-directory).

## HTTP security defaults

By default, Bastion binds to loopback (`127.0.0.1`) and is intended to be accessed locally.

If you want LAN access (no TLS), you must explicitly opt into insecure mode:

- `--insecure-http`
- or `BASTION_INSECURE_HTTP=1`

For public access, put Bastion behind a reverse proxy that terminates TLS.

See: [Reverse proxy](/user/operations/reverse-proxy).

## Precedence (CLI / env / database / defaults)

Different settings have different sources:

- **CLI flags**: highest priority
- **Environment variables**: next
- **Database**: only for fields that are managed by the Web UI runtime config page
- **Built-in defaults**: lowest priority

For runtime config fields shown in the Web UI:

1. CLI flags
2. Environment variables
3. Saved (database) value
4. Built-in default

See: [Runtime config](/user/operations/runtime-config).

Tip: `bastion config` shows effective values and where they come from.
