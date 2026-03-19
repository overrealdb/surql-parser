/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = ""Couldn't coerce value for field `id` of `log:bad`: Expected `[record, 'info' | 'warn' | 'error', datetime]` but found `'bad'`""

[[test.results]]
value = "[{ id: log:[user:one, 'info', d'2025-10-08T00:53:28.328764Z'], message: 'Database started' }]"
skip-datetime = true

*/

-- using multiple data types for a Complex Record ID
DEFINE FIELD id ON TABLE log TYPE [record, "info" | "warn" | "error", datetime];

-- Incorrect ID format, generates an error
CREATE log:bad SET level = "info", time = time::now(), message = "Database started";

-- Acceptable ID format
CREATE log:[user:one, "info", time::now()] SET message = "Database started";
