-- Rebuild `secrets` to support node-scoped credentials.
-- We use a NOT NULL `node_id` (reserved `hub`) so uniqueness works without NULL quirks.
CREATE TABLE IF NOT EXISTS secrets_new (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  kid INTEGER NOT NULL,
  nonce BLOB NOT NULL,
  ciphertext BLOB NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(kind, node_id, name)
);

INSERT INTO secrets_new (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at)
  SELECT id, 'hub', kind, name, kid, nonce, ciphertext, created_at, updated_at
  FROM secrets;

-- Rebuild `notification_destinations` to reference node-scoped secrets.
CREATE TABLE IF NOT EXISTS notification_destinations_new (
  node_id TEXT NOT NULL,
  secret_kind TEXT NOT NULL,
  secret_name TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (node_id, secret_kind, secret_name),
  FOREIGN KEY (secret_kind, node_id, secret_name) REFERENCES secrets_new(kind, node_id, name) ON DELETE CASCADE
);

INSERT INTO notification_destinations_new (node_id, secret_kind, secret_name, enabled, created_at, updated_at)
  SELECT 'hub', secret_kind, secret_name, enabled, created_at, updated_at
  FROM notification_destinations;

DROP TABLE notification_destinations;
DROP TABLE secrets;

ALTER TABLE secrets_new RENAME TO secrets;
ALTER TABLE notification_destinations_new RENAME TO notification_destinations;

CREATE INDEX IF NOT EXISTS idx_secrets_node_kind_name ON secrets(node_id, kind, name);
CREATE INDEX IF NOT EXISTS idx_notification_destinations_node_kind_enabled
  ON notification_destinations(node_id, secret_kind, enabled);
