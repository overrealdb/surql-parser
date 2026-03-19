/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: user:a856d1al1kiw5aasfu23, updated: d'1900-01-01T00:00:00Z' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: user:a856d1al1kiw5aasfu23, updated: d'1910-01-01T00:00:00Z' }]"
skip-record-id-key = true

*/

DEFINE FIELD updated ON TABLE user DEFAULT time::now();

-- Set `updated` to the year 1900
CREATE user SET updated = d"1900-01-01";
-- Then set to the year 1910
UPDATE user SET updated = d"1910-01-01";
