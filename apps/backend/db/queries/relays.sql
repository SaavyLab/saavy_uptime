-- name: insert_relay :exec
-- params: id String
-- params: slug String
-- params: name String
-- params: location_hint String
-- params: jurisdiction String
-- params: durable_object_id String
-- params: enabled i64
-- params: created_at i64
-- params: updated_at i64
-- params: last_bootstrapped_at Option<i64>
INSERT INTO relays (
  id,
  slug,
  name,
  location_hint,
  jurisdiction,
  durable_object_id,
  enabled,
  last_bootstrapped_at,
  created_at,
  updated_at
) VALUES (
  :id,
  :slug,
  :name,
  :location_hint,
  :jurisdiction,
  :durable_object_id,
  :enabled,
  :last_bootstrapped_at,
  :created_at,
  :updated_at
);

-- name: find_relay_by_slug :one
-- params: slug String
SELECT * FROM relays WHERE slug = :slug LIMIT 1;

-- name: find_relay_by_id :one
-- params: id String
SELECT * FROM relays WHERE id = :id LIMIT 1;

-- name: list_relays :many
SELECT * FROM relays ORDER BY created_at DESC;
