# Change: Refactor engine notifications module structure

## Why
`crates/bastion-engine/src/notifications.rs` is a large module mixing concerns (enqueue selection, worker loop scheduling, send logic per channel, and template rendering/context building). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Split engine notifications implementation into focused submodules under `crates/bastion-engine/src/notifications/`
- Keep the existing public surface stable (`notifications::enqueue_for_run_spec`, `notifications::spawn` and behavior)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/notifications.rs`, `crates/bastion-engine/src/notifications/*.rs`

## Compatibility / Non-Goals
- No changes intended to notification semantics, channel enablement rules, retry/backoff behavior, or template placeholder behavior.

