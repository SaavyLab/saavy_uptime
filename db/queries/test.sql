-- name: test_two :many
-- params: monitor_id String
-- returns: Vec<Monitor>
SELECT * 
FROM monitors 
WHERE id = :monitor_id;

-- name: multiline_with_white_space :scalar
SELECT *
FROM monitors

WHERE name = 'test';