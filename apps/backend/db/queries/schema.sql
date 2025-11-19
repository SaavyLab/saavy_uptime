CREATE INDEX idx_notifications_monitor_kind
  ON notifications (monitor_id, kind)

CREATE INDEX idx_organizations_owner_id ON organizations (owner_id)

CREATE TABLE incidents (
  id TEXT PRIMARY KEY,
  monitor_id TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
  opened_ts INTEGER NOT NULL,
  closed_ts INTEGER,
  reason TEXT,
  status TEXT NOT NULL DEFAULT 'open', -- open, acknowledged, closed
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
)

CREATE TABLE members (
  identity_id TEXT NOT NULL UNIQUE PRIMARY KEY, -- references sub from CF Access JWT
  email TEXT NOT NULL,
  is_workspace_admin INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
)

CREATE TABLE monitor_dispatches (
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
)

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
)

CREATE TABLE notifications (
  id TEXT PRIMARY KEY,
  monitor_id TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  target TEXT NOT NULL,
  config_json TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
)

CREATE TABLE organization_members (
  organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  identity_id TEXT NOT NULL, -- intentionally not a foreign key to allow for deletion of members from the organization
  role TEXT NOT NULL, -- admin, member
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,

  PRIMARY KEY (organization_id, identity_id)
)

CREATE TABLE organizations (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL, -- Display name
  created_at INTEGER NOT NULL
, owner_id TEXT NOT NULL REFERENCES members(identity_id) ON DELETE CASCADE)

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL
)
