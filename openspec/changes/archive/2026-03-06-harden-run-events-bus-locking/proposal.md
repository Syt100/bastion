# Change: Harden RunEventsBus locking against mutex poisoning

## Why
`RunEventsBus` uses a `std::sync::Mutex` and currently assumes it is never poisoned (via `expect`). A panic while holding the mutex would poison it, and subsequent use would panic the entire process. This is a reliability risk for a long-running Hub.

## What Changes
- Handle poisoned mutex locks without panicking (recover inner state and continue).
- Add a regression test that simulates mutex poisoning and validates the bus remains usable.
- Keep behavior and API stable (no protocol or schema changes).

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/run_events_bus.rs`

