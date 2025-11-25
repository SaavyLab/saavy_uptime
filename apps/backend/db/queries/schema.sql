CREATE INDEX idx_monitor_dispatch_hot_org_status
  ON monitor_dispatch_hot (org_id, status)

CREATE INDEX idx_monitors_org_enabled_next_run
  ON monitors (org_id, enabled, next_run_at)

CREATE INDEX idx_monitors_relay ON monitors(relay_id)

CREATE INDEX idx_monitors_status
  ON monitors (status)

CREATE INDEX idx_notifications_monitor_kind
  ON notifications (monitor_id, kind)

CREATE INDEX idx_organizations_owner_id ON organizations (owner_id)

CREATE INDEX idx_relays_enabled ON relays (enabled)

CREATE INDEX idx_relays_location ON relays (location_hint)

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
)

CREATE TABLE "monitors" (
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
, relay_id TEXT REFERENCES relays(id))

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
, owner_id TEXT NOT NULL REFERENCES members(identity_id) ON DELETE CASCADE, ae_sample_rate REAL NOT NULL DEFAULT 1.0)

CREATE TABLE relays (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  location_hint TEXT NOT NULL,
  jurisdiction TEXT NOT NULL,
  durable_object_id TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  last_bootstrapped_at INTEGER,
  last_error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK (length(durable_object_id) = 64)
)

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL
)
