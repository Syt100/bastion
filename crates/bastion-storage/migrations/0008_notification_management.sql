-- Key/value settings store (JSON values).
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

-- Notification destination metadata keyed by (secret kind, secret name).
-- Note: WeCom destinations use `secret_kind = 'wecom_bot'`.
--       Email destinations use `secret_kind = 'smtp'`.
CREATE TABLE IF NOT EXISTS notification_destinations (
  secret_kind TEXT NOT NULL,
  secret_name TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (secret_kind, secret_name),
  FOREIGN KEY (secret_kind, secret_name) REFERENCES secrets(kind, name) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notification_destinations_kind_enabled
  ON notification_destinations(secret_kind, enabled);

-- Seed destination metadata for existing secrets.
INSERT OR IGNORE INTO notification_destinations (secret_kind, secret_name, enabled, created_at, updated_at)
  SELECT kind, name, 1, updated_at, updated_at
  FROM secrets
  WHERE kind IN ('wecom_bot', 'smtp');

-- Seed default notifications settings.
INSERT OR IGNORE INTO settings (key, value_json, updated_at) VALUES (
  'notifications',
  '{
    "enabled": true,
    "channels": {
      "wecom_bot": { "enabled": true },
      "email": { "enabled": true }
    },
    "templates": {
      "wecom_markdown": "**{{title}}**\\n> Job: {{job_name}}\\n> Run: {{run_id}}\\n> Started: {{started_at}}\\n> Ended: {{ended_at}}\\n{{target_line_wecom}}{{error_line_wecom}}",
      "email_subject": "Bastion {{status_text}} - {{job_name}}",
      "email_body": "Bastion backup\\n\\nJob: {{job_name}}\\nRun: {{run_id}}\\nStatus: {{status}}\\nStarted: {{started_at}}\\nEnded: {{ended_at}}\\n{{target_line_email}}{{error_line_email}}"
    }
  }',
  CAST(strftime('%s','now') AS INTEGER)
);

