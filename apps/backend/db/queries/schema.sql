CREATE TABLE heartbeats (
  monitor_id TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
  ts INTEGER NOT NULL,
  ok INTEGER NOT NULL,
  code INTEGER,
  rtt_ms INTEGER,
  err TEXT,
  region TEXT, dispatch_id TEXT REFERENCES monitor_dispatches(id) ON DELETE SET NULL,
  PRIMARY KEY (monitor_id, ts)
)

CREATE INDEX idx_heartbeats_monitor_ts
  ON heartbeats (monitor_id, ts DESC)

CREATE INDEX idx_heartbeats_ts_only
  ON heartbeats (ts DESC)

CREATE INDEX idx_incidents_monitor_opened
  ON incidents (monitor_id, opened_ts DESC)

CREATE INDEX idx_incidents_status
  ON incidents (status, monitor_id) WHERE status = 'open'

CREATE INDEX idx_monitor_dispatches_monitor
  ON monitor_dispatches (monitor_id, created_at DESC)

CREATE INDEX idx_monitors_current_status
  ON monitors (current_status)

CREATE INDEX idx_monitors_org_enabled_next_run
  ON monitors (org_id, enabled, next_run_at_ts)

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
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'http',
  url TEXT NOT NULL,
  interval_s INTEGER NOT NULL,
  timeout_ms INTEGER NOT NULL,
  follow_redirects INTEGER NOT NULL DEFAULT 1,
  verify_tls INTEGER NOT NULL DEFAULT 1,
  expect_status_low INTEGER,
  expect_status_high INTEGER,
  expect_substring TEXT,
  headers_json TEXT,
  tags_json TEXT,
  enabled INTEGER NOT NULL DEFAULT 1,
  last_checked_at_ts INTEGER,
  next_run_at_ts INTEGER,

  current_status TEXT NOT NULL DEFAULT 'unknown', -- up, down, degraded, maintenance, unknown
  last_ok INTEGER NOT NULL DEFAULT 1, -- 1 = last check was ok, 0 = last check was not ok
  consecutive_failures INTEGER NOT NULL DEFAULT 0, -- number of consecutive failures
  current_incident_id TEXT REFERENCES incidents(id) ON DELETE SET NULL,

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
