-- Migration number: 0009 	 2025-11-20T00:05:06.214Z
PRAGMA defer_foreign_keys = true;

DROP TABLE IF EXISTS monitor_dispatches;

CREATE TABLE monitor_dispatch_hot (
  monitor_id TEXT PRIMARY KEY REFERENCES monitors(id) ON DELETE CASCADE,
  dispatch_id TEXT NOT NULL,
  org_id TEXT NOT NULL,
  status TEXT NOT NULL,
  scheduled_for_ts INTEGER NOT NULL,
  dispatched_at_ts INTEGER,
  completed_at_ts INTEGER,
  runner_colo TEXT,
  error TEXT,
  updated_at INTEGER NOT NULL
);

CREATE INDEX idx_monitor_dispatch_hot_org_status
  ON monitor_dispatch_hot (org_id, status);
