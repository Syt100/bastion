# Change: Update backend logging (defaults + file rotation)

## Why
The backend currently relies on `RUST_LOG` for log filtering, which can result in no visible logs by default and makes troubleshooting harder.

## What Changes
- Default backend log visibility to `INFO` without requiring `RUST_LOG`
- Add optional log-to-file output with configurable location and rotation
- Add targeted logs across critical workflows (scheduler/runs, targets, restore/verify, agent, notifications)
- Document logging configuration and redaction rules

## Impact
- Affected specs: `observability`
- Affected code: `crates/bastion/src/main.rs`, `crates/bastion/src/config.rs`, backend modules that perform runs/targets/restore/notify
