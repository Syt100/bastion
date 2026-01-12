## ADDED Requirements

### Requirement: Jobs Persist Schedule Timezone
The backend SHALL persist an IANA `schedule_timezone` per job, and SHALL expose it via authenticated job APIs.

#### Scenario: Create a job with explicit timezone
- **WHEN** a job is created with `schedule_timezone=Asia/Shanghai`
- **THEN** subsequent reads of the job return the same `schedule_timezone`

### Requirement: Hub Exposes Default Timezone
The backend SHALL expose the Hub timezone via `/api/system` so the UI can default new job schedules.

#### Scenario: System status includes hub timezone
- **WHEN** a user requests `/api/system`
- **THEN** the response includes `hub_timezone`

### Requirement: Timezone-Aware Scheduling with DST Semantics
The scheduler SHALL interpret cron schedules in the job’s configured timezone, using wall-clock semantics.

#### Scenario: DST gap is skipped
- **GIVEN** a timezone with a DST spring-forward gap
- **WHEN** the missing local time would be due by cron
- **THEN** no run is enqueued for the nonexistent local time

#### Scenario: DST fold runs once
- **GIVEN** a timezone with a DST fall-back fold
- **WHEN** a cron is due at a repeated local time
- **THEN** exactly one run is enqueued for that local wall time

### Requirement: Cron Validation Avoids “Never Triggers”
The backend SHALL validate cron expressions and reject schedules that could never trigger under the minute-based scheduler loop.

#### Scenario: 6-field cron with seconds != 0 is rejected
- **WHEN** a user sets a 6-field cron where seconds is not `0`
- **THEN** the API returns `invalid_schedule`

### Requirement: Agent Offline Scheduling Uses Same Timezone Semantics
Agent config snapshots SHALL include `schedule_timezone` and offline scheduling SHALL use the same evaluation logic as the Hub scheduler.

#### Scenario: Snapshot includes schedule timezone
- **WHEN** the Hub sends a config snapshot for an agent job
- **THEN** the job config includes `schedule_timezone`

