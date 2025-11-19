-- name: create_organization :exec :stmt
-- params: id String
-- params: slug String
-- params: name String
-- params: owner_id String
-- params: created_at i64
INSERT INTO organizations (id, slug, name, owner_id, created_at) VALUES (:id, :slug, :name, :owner_id, :created_at);

-- name: get_organization_by_id :one
SELECT * FROM organizations WHERE id = :id;

-- name: check_if_bootstrapped :scalar
SELECT COUNT(*) as count FROM organizations;

-- name: select_all_org_ids :many
SELECT id FROM organizations;

-- name: select_org_member :one
SELECT organization_id, role from organization_members where identity_id = :identity_id ORDER BY created_at DESC LIMIT 1;

-- name: get_organization_members :many
select m.email, om.role from members m join organization_members om on m.identity_id = om.identity_id where om.organization_id = :organization_id;

-- name: get_org_sample_rate :one
SELECT id, ae_sample_rate FROM organizations WHERE id = :id;