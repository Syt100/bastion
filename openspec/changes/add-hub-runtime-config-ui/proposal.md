# Change: Add Hub Runtime Config UI (Restart Required)

## Why
Hub configuration is currently set via CLI args and environment variables, which makes it hard to audit and manage from the Web UI.
We want a Hub-only “runtime config” page that:
- Displays current effective config and the corresponding environment variable names.
- Allows editing a safe subset of config in the UI.
- Persists changes and clearly indicates they take effect only after a Hub restart.

## What Changes
- Backend:
  - Persist a Hub runtime config document in the DB settings table.
  - On startup, apply saved config values when not overridden by CLI/ENV.
  - Expose an authenticated API to read effective vs saved values, with per-field source (`cli|env|db|default`) and editability.
- Web UI:
  - Add a Settings page to view/edit Hub runtime config.
  - Show “requires restart” banners and per-field “pending” indicators when saved differs from effective.
  - Read-only display for unsafe fields (bind host/port, trusted proxies, insecure HTTP).

## Impact
- Affected specs: `control-plane`, `web-ui`
- Affected code: `crates/bastion`, `crates/bastion-http`, `crates/bastion-storage`, `ui`

