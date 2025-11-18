-- name: create_monitor :exec
-- params: id String
-- params: org_id String
-- params: name String
-- params: kind String
-- params: url String
-- params: interval_s i64
-- params: timeout_ms i64
-- params: follow_redirects i64
-- params: verify_tls i64
-- params: created_at i64
-- params: updated_at i64
INSERT INTO monitors (
    id, 
    org_id, 
    name, 
    kind, 
    url, 
    interval_s, 
    timeout_ms, 
    follow_redirects, 
    verify_tls, 
    created_at, 
    updated_at
) VALUES (
    :id, 
    :org_id, 
    :name, 
    :kind, 
    :url, 
    :interval_s, 
    :timeout_ms, 
    :follow_redirects, 
    :verify_tls, 
    :created_at, 
    :updated_at
);

-- name: get_monitor_by_id :one
SELECT * FROM monitors WHERE id = :id AND org_id = :org_id;

-- name: get_monitors_by_org_id :many
SELECT * FROM monitors WHERE org_id = :org_id ORDER BY created_at DESC;

-- name: delete_monitor :exec
DELETE FROM monitors WHERE id = :id AND org_id = :org_id;
