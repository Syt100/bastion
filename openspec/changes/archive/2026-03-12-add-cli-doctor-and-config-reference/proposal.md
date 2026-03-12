# Change: Add `bastion doctor` / `bastion config` and auto-generate config reference docs

## Why
For individuals/families/small teams, the main sources of friction are setup and troubleshooting:

- Knowing which configuration is actually in effect (CLI/ENV/DB/default) is hard without the Web UI.
- Common pitfalls (data dir permissions, missing UI/docs assets, reverse proxy headers) take time to diagnose.
- Documentation about configuration and environment variables can drift from the code.

We want a CLI-first way to:

- Inspect effective config and its sources.
- Run a quick health/diagnostics checklist ("doctor").
- Keep configuration reference docs in sync with the actual CLI/env definitions.

## What Changes
- Add `bastion config`:
  - Print effective Hub config values and the resolved source (`cli|env|db|default`), matching the semantics shown in the Web UI runtime config page.
  - Support `--json` for machine-readable output.
- Add `bastion doctor`:
  - Run diagnostics for common setup issues (data dir, DB connectivity, secrets/keyring access, embedded/static UI+docs assets availability, and basic config validity).
  - Exit non-zero on failures; print actionable guidance.
  - Support `--json` for CI/scripts.
- Docs automation:
  - Extend doc generation to output a config/environment reference page (English + Chinese) derived from the clap command tree plus product-specific env vars (e.g. UI/docs dir overrides).
  - CI fails if generated outputs are stale.

## Impact
- Affected specs: `cli`, `dev-workflow`
- Affected code:
  - `crates/bastion/src/config.rs`, `crates/bastion/src/main.rs`
  - `crates/bastion/src/bin/docgen.rs` (+ generated docs pages)
  - `docs/*` sidebar and reference pages

## Non-Goals
- A full remote troubleshooting workflow (this is local CLI diagnostics only).
- Modifying the Web UI runtime config model; this change only surfaces the same information via CLI.
- Adding more locales beyond `en-US` and `zh-CN`.

