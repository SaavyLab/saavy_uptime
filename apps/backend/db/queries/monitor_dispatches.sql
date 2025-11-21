-- name: upsert_dispatch_pending :exec
-- params: monitor_id String
-- params: dispatch_id String
-- params: org_id String
-- params: scheduled_for_ts i64
-- params: updated_at i64
INSERT INTO monitor_dispatch_hot (
  monitor_id,
  dispatch_id,
  org_id,
  status,
  scheduled_for_ts,
  dispatched_at_ts,
  completed_at_ts,
  runner_colo,
  error,
  updated_at
) VALUES (
  :monitor_id,
  :dispatch_id,
  :org_id,
  'pending',
  :scheduled_for_ts,
  NULL,
  NULL,
  NULL,
  NULL,
  :updated_at
)
ON CONFLICT(monitor_id) DO UPDATE SET
  dispatch_id = excluded.dispatch_id,
  org_id = excluded.org_id,
  status = 'pending',
  scheduled_for_ts = excluded.scheduled_for_ts,
  dispatched_at_ts = NULL,
  completed_at_ts = NULL,
  runner_colo = NULL,
  error = NULL,
  updated_at = excluded.updated_at;

-- name: mark_dispatch_running :exec
-- params: dispatched_at_ts i64
-- params: runner_colo Option<String>
-- params: updated_at i64
-- params: monitor_id String
-- params: dispatch_id String
UPDATE monitor_dispatch_hot
SET status = 'running',
    dispatched_at_ts = :dispatched_at_ts,
    runner_colo = :runner_colo,
    updated_at = :updated_at
WHERE monitor_id = :monitor_id
  AND dispatch_id = :dispatch_id;

-- name: finalize_dispatch :exec
-- params: status String
-- params: completed_at_ts i64
-- params: error Option<String>
-- params: updated_at i64
-- params: monitor_id String
-- params: dispatch_id String
UPDATE monitor_dispatch_hot
SET status = :status,
    completed_at_ts = :completed_at_ts,
    error = :error,
    updated_at = :updated_at
WHERE monitor_id = :monitor_id
  AND dispatch_id = :dispatch_id;