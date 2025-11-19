-- Migration number: 0005 	 2025-11-19T05:29:28.956Z
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