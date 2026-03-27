EXPLAIN ANALYZE SELECT
  id as commentId,
  in.id as id,
  in.creationDate as creationDate
FROM is_comment_of
WHERE out = media_text_test:0 AND in.creationDate < d'2026-01-09T00:00:00.000Z'
ORDER BY in.creationDate DESC
LIMIT 2;
