# Change: Improve upgrade/rollback docs, doctor checks, defaults, and observability

## Why
For individuals, families, and small teams, most “bad experiences” happen during setup, upgrades, and troubleshooting.

We should reduce surprise defaults, document safe upgrade/rollback procedures, and provide basic observability endpoints and CLI diagnostics.

## What Changes
- Docs: add an **Upgrade & rollback** guide (EN/ZH) covering Docker, Linux packages (systemd), Windows MSI service, and portable tar/zip installs.
  - Emphasize data directory backup and the fact that DB migrations may not be reversible.
- CLI: enhance `bastion doctor` to catch more common operational issues and provide actionable guidance.
  - Keep `--json` output stable for scripts/CI.
- Defaults: document and surface default behavior and configuration precedence (CLI/env/DB/default) more clearly.
- Observability: define and document liveness/readiness semantics and expose endpoints suitable for service managers and simple probes.

## Impact
- Affected specs: `cli`, `dev-workflow`, `observability`
- Affected code:
  - `crates/bastion/src/main.rs` (doctor enhancements)
  - `crates/bastion-http/src/http/mod.rs` (readiness endpoint)
  - Docs under `docs/` (new operations pages, EN/ZH)

## Non-Goals
- Full Prometheus/OpenTelemetry metrics pipeline.
- Automated in-place rollback tooling (we document manual rollback).
