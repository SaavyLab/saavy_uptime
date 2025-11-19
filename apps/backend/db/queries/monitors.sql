-- name: create_monitor :exec
-- params: id String
-- params: org_id String
-- params: name String
-- params: kind String
-- params: enabled i64
-- params: config_json String
-- params: status String
-- params: created_at i64
-- params: updated_at i64
INSERT INTO monitors (
    id, 
    org_id, 
    name, 
    kind, 
    enabled,
    config_json,
    status,
    created_at, 
    updated_at
) VALUES (
    :id, 
    :org_id, 
    :name, 
    :kind, 
    :enabled,
    :config_json,
    :status,
    :created_at, 
    :updated_at
);

-- name: get_monitor_by_id :one
SELECT * FROM monitors WHERE id = :id AND org_id = :org_id;

-- name: get_monitors_by_org_id :many
SELECT * FROM monitors WHERE org_id = :org_id ORDER BY created_at DESC;

-- name: delete_monitor :exec
DELETE FROM monitors WHERE id = :id AND org_id = :org_id;

-- name: update_monitor_status :exec
-- params: id String
-- params: org_id String
-- params: status String
-- params: last_checked_at i64
-- params: last_failed_at i64
-- params: first_checked_at i64
-- params: rt_ms i64
-- params: last_error String
-- params: updated_at i64
UPDATE monitors SET status = :status, last_checked_at = :last_checked_at, last_failed_at = :last_failed_at, first_checked_at = :first_checked_at, rt_ms = :rt_ms, last_error = :last_error, updated_at = :updated_at WHERE id = :id AND org_id = :org_id;
