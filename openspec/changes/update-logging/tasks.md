## 1. Spec
- [x] 1.1 Add `observability` spec delta for default INFO logging and file rotation
- [x] 1.2 Run `openspec validate update-logging --strict`

## 2. Backend
- [x] 2.1 Add CLI/env config for logging (filter + optional file output + rotation + retention)
- [x] 2.2 Initialize `tracing` with default INFO visibility and conservative defaults for noisy deps
- [x] 2.3 Implement optional rotating file logging and retention cleanup
- [x] 2.4 Add targeted logs to critical workflows (runs/scheduler, targets, restore/verify, agent, notifications)
- [x] 2.5 Add/adjust tests for logging config / retention cleanup behavior

## 3. Docs
- [x] 3.1 Document logging configuration and redaction guidance

## 4. Validation
- [x] 4.1 Run `cargo fmt`, `cargo clippy`, `cargo test`
