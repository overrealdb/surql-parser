/**[test]

[[test.results]]
value = "NONE"

*/

-- By marking a table as DROP, you disallow any records to be created or updated.
-- Records that currently exist in the table will not automatically be deleted, you can still remove them manually.
DEFINE TABLE reading DROP;
