-- Link operations to a subject entity (e.g. a backup run) so UIs can show related work together.
ALTER TABLE operations ADD COLUMN subject_kind TEXT;
ALTER TABLE operations ADD COLUMN subject_id TEXT;

CREATE INDEX IF NOT EXISTS idx_operations_subject_started_at
  ON operations(subject_kind, subject_id, started_at);

