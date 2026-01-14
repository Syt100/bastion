-- Agent labels (tags) for multi-node grouping and bulk selection.
CREATE TABLE IF NOT EXISTS agent_labels (
  agent_id TEXT NOT NULL,
  label TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (agent_id, label),
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_agent_labels_label ON agent_labels(label);

