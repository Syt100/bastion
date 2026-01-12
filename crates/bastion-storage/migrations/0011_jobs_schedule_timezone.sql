-- Per-job schedule timezone (IANA), used to interpret cron schedules as wall-clock time.
-- Default to UTC to avoid NULLs and ensure deterministic behavior.
ALTER TABLE jobs ADD COLUMN schedule_timezone TEXT NOT NULL DEFAULT 'UTC';
CREATE INDEX IF NOT EXISTS idx_jobs_schedule_timezone ON jobs(schedule_timezone);

