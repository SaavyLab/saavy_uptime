-- name: get_organization_by_id :one
SELECT * FROM organizations WHERE id = :id;

-- name: create_organization :exec
-- instrument: skip(name)
INSERT INTO organizations (id, slug, name) VALUES (:id, :slug, :name);

-- name: check_if_bootstrapped :scalar
SELECT COUNT(*) as count FROM organizations;

-- name: select_all_org_ids :many
SELECT id FROM organizations;

-- name: select_org_member :one
SELECT organization_id, role from organization_members where identity_id = :identity_id ORDER BY created_at DESC LIMIT 1;