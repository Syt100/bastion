-- Optional association of jobs to a specific Agent (remote node).
ALTER TABLE jobs ADD COLUMN agent_id TEXT;
CREATE INDEX IF NOT EXISTS idx_jobs_agent_id ON jobs(agent_id);

