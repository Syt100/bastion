-- Snapshot index for successful backup runs ("backup data management").
--
-- A snapshot is identified by run_id and records the run-time target snapshot
-- so future lifecycle actions (delete/retention) do not depend on the current job spec.

CREATE TABLE IF NOT EXISTS run_artifacts (
  run_id TEXT PRIMARY KEY,
  job_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_snapshot_json TEXT NOT NULL,
  artifact_format TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  ended_at INTEGER NOT NULL,
  source_files INTEGER,
  source_dirs INTEGER,
  source_bytes INTEGER,
  transfer_bytes INTEGER,
  last_error_kind TEXT,
  last_error TEXT,
  last_attempt_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE,
  FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_run_artifacts_job_ended_at ON run_artifacts(job_id, ended_at);
CREATE INDEX IF NOT EXISTS idx_run_artifacts_job_status ON run_artifacts(job_id, status);
CREATE INDEX IF NOT EXISTS idx_run_artifacts_node_status ON run_artifacts(node_id, status);

