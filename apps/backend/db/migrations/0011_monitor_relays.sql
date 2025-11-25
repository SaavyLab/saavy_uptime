-- Migration number: 0011 	 2025-11-25T21:53:13.390Z
PRAGMA defer_foreign_keys = true;

ALTER TABLE monitors ADD COLUMN relay_id TEXT REFERENCES relays(id);

CREATE INDEX IF NOT EXISTS idx_monitors_relay ON monitors(relay_id);
