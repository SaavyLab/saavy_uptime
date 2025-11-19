-- Migration number: 0007 	 2025-11-19T19:28:24.369Z

PRAGMA defer_foreign_keys = true;

CREATE TABLE monitors_v7 (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'http',
  enabled INTEGER NOT NULL DEFAULT 1,
  config_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  last_checked_at INTEGER,
  last_failed_at INTEGER,
  first_checked_at INTEGER,
  rt_ms INTEGER,
  region TEXT,
  last_error TEXT,
  next_run_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

INSERT INTO monitors_v7 (
  id,
  org_id,
  name,
  kind,
  enabled,
  config_json,
  status,
  last_checked_at,
  last_failed_at,
  first_checked_at,
  rt_ms,
  region,
  last_error,
  next_run_at,
  created_at,
  updated_at
)
SELECT
  id,
  org_id,
  name,
  kind,
  enabled,
  config_json,
  lower(status) AS status,
  last_checked_at,
  last_failed_at,
  first_checked_at,
  rt_ms,
  region,
  last_error,
  next_run_at,
  created_at,
  updated_at
FROM monitors;

DROP TABLE monitors;

ALTER TABLE monitors_v7 RENAME TO monitors;

CREATE INDEX IF NOT EXISTS idx_monitors_org_enabled_next_run
  ON monitors (org_id, enabled, next_run_at);

CREATE INDEX IF NOT EXISTS idx_monitors_status
  ON monitors (status);
