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
SELECT id, org_id, name, kind, enabled, config_json, status, last_checked_at, last_failed_at, first_checked_at, rt_ms, region, relay_id, last_error, next_run_at, created_at, updated_at FROM monitors WHERE id = :id AND org_id = :org_id;

-- name: get_monitors_by_org_id :many
SELECT id, org_id, name, kind, enabled, config_json, status, last_checked_at, last_failed_at, first_checked_at, rt_ms, region, relay_id, last_error, next_run_at, created_at, updated_at FROM monitors WHERE org_id = :org_id ORDER BY created_at DESC;

-- name: list_due_monitors :many
-- params: org_id String
-- params: next_run_at Option<i64>
-- params: limit i64
SELECT id, kind, config_json, status, first_checked_at, last_failed_at, next_run_at, relay_id
FROM monitors
WHERE org_id = :org_id
  AND enabled = 1
  AND (
        :next_run_at IS NULL
        OR next_run_at IS NULL
        OR next_run_at <= :next_run_at
      )
ORDER BY COALESCE(next_run_at, 0) ASC
LIMIT :limit;

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

-- name: update_monitor_next_run_at :exec :stmt
-- params: id String
-- params: org_id String
-- params: next_run_at i64
-- params: last_checked_at i64
-- params: updated_at i64
UPDATE monitors SET next_run_at = :next_run_at, last_checked_at = :last_checked_at, updated_at = :updated_at WHERE id = :id AND org_id = :org_id;

-- name: set_monitor_relay :exec
-- params: id String
-- params: org_id String
-- params: relay_id String
-- params: updated_at i64
UPDATE monitors SET relay_id = :relay_id, updated_at = :updated_at WHERE id = :id AND org_id = :org_id;