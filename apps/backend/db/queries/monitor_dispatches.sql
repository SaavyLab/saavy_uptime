-- name: dispatch_monitor :exec
-- params: status String
-- params: dispatched_at_ts i64
-- params: id String
UPDATE monitor_dispatches SET status = :status, dispatched_at_ts = :dispatched_at_ts WHERE id = :id;

-- name: complete_dispatch :exec
-- params: status String
-- params: completed_at_ts i64
-- params: id String
UPDATE monitor_dispatches SET status = :status, completed_at_ts = :completed_at_ts WHERE id = :id;

-- name: create_dispatch :exec
-- params: id String
-- params: monitor_id String
-- params: org_id String
-- params: status String
-- params: scheduled_for_ts i64
-- params: created_at i64
INSERT INTO monitor_dispatches (id, monitor_id, org_id, status, scheduled_for_ts, created_at) VALUES (:id, :monitor_id, :org_id, :status, :scheduled_for_ts, :created_at);