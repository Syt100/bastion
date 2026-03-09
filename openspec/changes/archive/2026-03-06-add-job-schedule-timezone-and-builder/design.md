## Context
Scheduling currently uses a cron string stored on jobs and evaluated by:
- The Hub scheduler when the agent is connected.
- The agent offline scheduler when disconnected.

Both paths evaluate schedules using UTC timestamps and a minute-based loop.

This change introduces explicit timezone semantics while keeping the internal execution pipeline (runs, queueing, workers) in UTC.

## Goals / Non-Goals
### Goals
- Make scheduling intuitive for users by supporting a guided schedule builder plus raw cron.
- Make schedule timezone explicit per job and consistent between Hub and offline agent scheduling.
- Define DST behavior that avoids surprising double-runs.

### Non-Goals
- Cross-version compatibility guarantees (project not launched).
- Supporting non-standard cron aliases (e.g. `@daily`).

## Decisions
### Decision: Store timezone separately from cron
- Add `jobs.schedule_timezone` as an IANA timezone string.
- Store cron as a 5-field expression in UI; backend supports 5-field and 6-field (seconds) but only with `seconds == 0`.

### Decision: Interpret schedules as wall-clock in the configured timezone
- Each evaluation converts “now/minute_start (UTC)” → “local time (job timezone)” and checks whether the cron matches that local minute.
- The resulting run enqueue/creation remains in UTC.

### Decision: DST semantics
- **Gap (spring forward, nonexistent local times)**: skip the missing local times (no “make-up” run).
- **Fold (fall back, ambiguous local times)**: run once by selecting the *first* occurrence (earlier offset).

### Decision: Hub timezone default
- Add a Hub timezone config value (IANA name) exposed by `/api/system`.
- The job editor defaults `schedule_timezone` to the Hub timezone.

## Alternatives Considered
- Convert cron strings to a single UTC cron at creation time: rejected because DST makes this non-equivalent.
- Always schedule in UTC: rejected due to user expectation mismatch (“daily at 02:00 local”).

## Risks / Trade-offs
- Timezone selection adds UI complexity; mitigated by a default and searchable selector.
- DST behavior must be documented; mitigated by consistent defaults (gap=skip, fold=run once) and clear UI hints.

