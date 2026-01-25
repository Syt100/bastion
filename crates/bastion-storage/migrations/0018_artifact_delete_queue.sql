-- Backup snapshot deletion queue ("backup data management").
--
-- Deleting a snapshot can involve remote targets (WebDAV) and should be:
-- - asynchronous (avoid HTTP timeouts)
-- - idempotent (repeated requests are safe)
-- - observable (status + attempts + event log)

CREATE TABLE IF NOT EXISTS artifact_delete_tasks (
  run_id TEXT PRIMARY KEY,
  job_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_snapshot_json TEXT NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  last_attempt_at INTEGER,
  next_attempt_at INTEGER NOT NULL,
  last_error_kind TEXT,
  last_error TEXT,
  ignored_at INTEGER,
  ignored_by_user_id INTEGER,
  ignore_reason TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE,
  FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
  FOREIGN KEY (ignored_by_user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_artifact_delete_tasks_status_next_attempt
  ON artifact_delete_tasks(status, next_attempt_at);
CREATE INDEX IF NOT EXISTS idx_artifact_delete_tasks_job_id ON artifact_delete_tasks(job_id);
CREATE INDEX IF NOT EXISTS idx_artifact_delete_tasks_node_id ON artifact_delete_tasks(node_id);

CREATE TABLE IF NOT EXISTS artifact_delete_events (
  run_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  ts INTEGER NOT NULL,
  level TEXT NOT NULL,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  fields_json TEXT,
  PRIMARY KEY (run_id, seq),
  FOREIGN KEY (run_id) REFERENCES artifact_delete_tasks(run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_artifact_delete_events_ts ON artifact_delete_events(ts);

