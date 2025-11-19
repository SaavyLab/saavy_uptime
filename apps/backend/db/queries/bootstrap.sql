-- name: create_organization :exec :stmt
-- params: id String
-- params: slug String
-- params: name String
-- params: owner_id String
-- params: created_at i64
INSERT INTO organizations (id, slug, name, owner_id, created_at) VALUES (:id, :slug, :name, :owner_id, :created_at);

-- name: create_member :exec :stmt
-- params: identity_id String
-- params: email String
-- params: is_workspace_admin i64
-- params: created_at i64
-- params: updated_at i64
INSERT INTO members (identity_id, email, is_workspace_admin, created_at, updated_at) VALUES (:identity_id, :email, :is_workspace_admin, :created_at, :updated_at)
ON CONFLICT(identity_id) DO UPDATE SET email=excluded.email, updated_at=excluded.updated_at;

-- name: create_organization_member :exec :stmt
-- params: organization_id String
-- params: identity_id String
-- params: role String
-- params: created_at i64
-- params: updated_at i64
INSERT INTO organization_members (organization_id, identity_id, role, created_at, updated_at) VALUES (:organization_id, :identity_id, :role, :created_at, :updated_at);