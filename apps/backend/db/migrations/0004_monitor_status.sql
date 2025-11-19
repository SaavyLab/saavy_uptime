-- Migration number: 0004 	 2025-11-19T05:19:57.437Z

-- 1. Drop old table (if empty) or Rename
-- Let's Rename to migrate data if you have it, otherwise Drop.
-- Assuming clean slate or migration script handling data migration:
DROP TABLE IF EXISTS monitors;
DROP TABLE IF EXISTS monitor_dispatches;
DROP TABLE IF EXISTS heartbeats;
DROP TABLE IF EXISTS incidents;

CREATE TABLE monitors (
  -- Identity & Ownership
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  
  -- Core Config (Queryable)
  name TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'http', -- 'http', 'tcp', 'udp'
  enabled INTEGER NOT NULL DEFAULT 1,
  
  -- Execution Config (The "How")
  -- Stores: url, interval, timeout, headers, expect_status, etc.
  -- Why: The Dispatcher just grabs this JSON and passes it to the runner.
  config_json TEXT NOT NULL, 

  -- The "Hot State" (Updated by Consumer)
  status TEXT NOT NULL DEFAULT 'PENDING', -- 'UP', 'DOWN', 'DEGRADED'
  last_checked_at INTEGER,     -- timestamp ms
  last_failed_at INTEGER,      -- timestamp ms (The Anchor for streaks)
  first_checked_at INTEGER,    -- timestamp ms (The Anchor for uptime)
  rt_ms INTEGER,               -- Latest latency
  region TEXT,                 -- Latest colo/region
  last_error TEXT,             -- Human readable error for tooltip
  
  -- Scheduling
  next_run_at INTEGER,         -- Used by Ticker DO

  -- Meta
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE incidents (
  id TEXT PRIMARY KEY,
  monitor_id TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
  opened_ts INTEGER NOT NULL,
  closed_ts INTEGER,
  reason TEXT,
  status TEXT NOT NULL DEFAULT 'open', -- open, acknowledged, closed
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);