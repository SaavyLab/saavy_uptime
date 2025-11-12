PRAGMA defer_foreign_keys = true;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS monitor_dispatches (
  id TEXT PRIMARY KEY,
  monitor_id TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
  org_id TEXT NOT NULL,
  status TEXT NOT NULL,
  scheduled_for_ts INTEGER NOT NULL,
  dispatched_at_ts INTEGER,
  completed_at_ts INTEGER,
  runner_colo TEXT,
  error TEXT,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_monitor_dispatches_monitor
  ON monitor_dispatches (monitor_id, created_at DESC);

ALTER TABLE heartbeats
  ADD COLUMN dispatch_id TEXT REFERENCES monitor_dispatches(id) ON DELETE SET NULL;

COMMIT;
