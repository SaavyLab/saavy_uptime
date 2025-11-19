-- Migration number: 0006 	 2025-11-19T18:58:11.757Z
CREATE INDEX idx_monitors_org_enabled_next_run ON monitors (org_id, enabled, next_run_at);
CREATE INDEX idx_monitors_status ON monitors (status);