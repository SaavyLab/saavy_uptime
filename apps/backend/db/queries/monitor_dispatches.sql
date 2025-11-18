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