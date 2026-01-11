-- Incomplete cleanup management: job archive, run target snapshots, and persistent cleanup queue.

-- Jobs can be archived (soft-delete) so history remains but scheduling stops.
ALTER TABLE jobs ADD COLUMN archived_at INTEGER;

-- Runs store a target snapshot used by maintenance workflows.
ALTER TABLE runs ADD COLUMN target_snapshot_json TEXT;

-- Persistent incomplete cleanup task queue.
CREATE TABLE IF NOT EXISTS incomplete_cleanup_tasks (
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
  FOREIGN KEY (ignored_by_user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_incomplete_cleanup_tasks_status_next_attempt
  ON incomplete_cleanup_tasks(status, next_attempt_at);
CREATE INDEX IF NOT EXISTS idx_incomplete_cleanup_tasks_job_id ON incomplete_cleanup_tasks(job_id);
CREATE INDEX IF NOT EXISTS idx_incomplete_cleanup_tasks_node_id ON incomplete_cleanup_tasks(node_id);

-- Audit trail for cleanup attempts and operator actions.
CREATE TABLE IF NOT EXISTS incomplete_cleanup_events (
  run_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  ts INTEGER NOT NULL,
  level TEXT NOT NULL,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  fields_json TEXT,
  PRIMARY KEY (run_id, seq),
  FOREIGN KEY (run_id) REFERENCES incomplete_cleanup_tasks(run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_incomplete_cleanup_events_ts ON incomplete_cleanup_events(ts);

