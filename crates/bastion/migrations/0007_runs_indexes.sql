-- Indexes for hot query paths on runs.
CREATE INDEX IF NOT EXISTS idx_runs_status_started_at ON runs(status, started_at);
CREATE INDEX IF NOT EXISTS idx_runs_ended_at ON runs(ended_at);

