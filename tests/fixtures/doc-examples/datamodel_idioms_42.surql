-- Parse error: what is the `.` referring to?
-- DB: "Call recursive query on a `planet`? Its `name` field? Something else?"
SELECT .{1..3}(->has->(?)) FROM planet;

-- A similar query that can't be parsed
-- DB: "Call .len() on what?"
SELECT .len() FROM planet;
