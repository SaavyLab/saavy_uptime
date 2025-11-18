-- name: write_heartbeat :exec
-- params: monitor_id String
-- params: dispatch_id String
-- params: ts i64
-- params: ok i64
-- params: code i64
-- params: rtt_ms i64
-- params: err String
-- params: region String
INSERT INTO heartbeats (monitor_id, dispatch_id, ts, ok, code, rtt_ms, err, region) VALUES (:monitor_id, :dispatch_id, :ts, :ok, :code, :rtt_ms, :err, :region);

-- name: get_heartbeats_by_monitor_id :many
-- params: org_id String
-- params: monitor_id String
-- params: before i64
-- params: limit i64
SELECT h.*, m.org_id FROM heartbeats h
INNER JOIN monitors m ON h.monitor_id = m.id
WHERE m.org_id = :org_id AND h.monitor_id = :monitor_id AND h.ts < :before
ORDER BY h.ts DESC
LIMIT :limit