-- Bulk operations framework: persistent, async, per-node items.
CREATE TABLE IF NOT EXISTS bulk_operations (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  created_by_user_id INTEGER,
  selector_json TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  started_at INTEGER,
  ended_at INTEGER,
  canceled_at INTEGER,
  FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_bulk_operations_status_created_at
  ON bulk_operations(status, created_at);

CREATE TABLE IF NOT EXISTS bulk_operation_items (
  op_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  started_at INTEGER,
  ended_at INTEGER,
  last_error_kind TEXT,
  last_error TEXT,
  PRIMARY KEY (op_id, agent_id),
  FOREIGN KEY (op_id) REFERENCES bulk_operations(id) ON DELETE CASCADE,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_bulk_operation_items_status_updated_at
  ON bulk_operation_items(status, updated_at);
CREATE INDEX IF NOT EXISTS idx_bulk_operation_items_op_id_status
  ON bulk_operation_items(op_id, status);

