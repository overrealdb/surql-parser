-- Will now call `.{1..3}(has->(?))` on every planet record it finds
SELECT @.{1..3}(->has->(?)) AS cities FROM planet;
-- Will now call `.len()` on every `name` field it finds
SELECT name.len()           AS length FROM planet;
