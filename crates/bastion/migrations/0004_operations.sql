-- Long-running operations such as restore and restore drill verification.
CREATE TABLE IF NOT EXISTS operations (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  summary_json TEXT,
  error TEXT
);

CREATE INDEX IF NOT EXISTS idx_operations_status_started_at ON operations(status, started_at);

CREATE TABLE IF NOT EXISTS operation_events (
  op_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  ts INTEGER NOT NULL,
  level TEXT NOT NULL,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  fields_json TEXT,
  PRIMARY KEY (op_id, seq),
  FOREIGN KEY (op_id) REFERENCES operations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_operation_events_ts ON operation_events(ts);

