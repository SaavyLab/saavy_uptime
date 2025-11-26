-- Migration number: 0010 	 2025-11-25T21:26:53.903Z
PRAGMA defer_foreign_keys = true;

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
);

CREATE INDEX idx_relays_enabled ON relays (enabled);
CREATE INDEX idx_relays_location ON relays (location_hint);
