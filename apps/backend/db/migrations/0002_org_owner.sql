-- Migration number: 0002 	 2025-11-11T18:27:08.050Z


CREATE TABLE IF NOT EXISTS members (
  identity_id TEXT NOT NULL UNIQUE PRIMARY KEY, -- references sub from CF Access JWT
  email TEXT NOT NULL,
  is_workspace_admin INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

ALTER TABLE organizations ADD COLUMN owner_id TEXT NOT NULL REFERENCES members(identity_id) ON DELETE CASCADE; -- references sub from CF Access JWT

CREATE INDEX IF NOT EXISTS idx_organizations_owner_id ON organizations (owner_id);

CREATE TABLE IF NOT EXISTS organization_members (
  organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  identity_id TEXT NOT NULL, -- intentionally not a foreign key to allow for deletion of members from the organization
  role TEXT NOT NULL, -- admin, member
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,

  PRIMARY KEY (organization_id, identity_id)
);
