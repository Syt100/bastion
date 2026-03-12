## ADDED Requirements

### Requirement: Runtime Log Filter Resolution Is Explicit-Input Driven
Backend runtime log filter resolution SHALL derive `RUST_LOG` fallback behavior from explicit caller-provided input rather than reading mutable process environment from resolution helpers.

#### Scenario: Runtime entry point supplies captured RUST_LOG
- **GIVEN** a backend runtime entry point needs to resolve effective logging configuration
- **WHEN** `RUST_LOG` is set in the process environment
- **THEN** the entry point captures that value and passes it into runtime-config resolution explicitly
- **AND** downstream resolution helpers do not read `RUST_LOG` directly

### Requirement: Runtime Config Resolution Tests Are Parallel-Safe
Backend unit tests covering runtime config log-filter precedence SHALL remain deterministic when the Rust test runner executes tests in parallel.

#### Scenario: DB fallback test runs alongside env-precedence test
- **GIVEN** one test validates `RUST_LOG` precedence and another validates DB fallback behavior
- **WHEN** the backend test binary runs with parallel test threads
- **THEN** each test controls its own log-filter inputs explicitly
- **AND** the DB fallback assertion does not observe leaked process-global `RUST_LOG` state
