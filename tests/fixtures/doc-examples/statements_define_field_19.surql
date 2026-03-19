/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: user:059nm0eomnrwvq692i3i, updated: d'2025-10-07T07:48:06.852628Z' }]"
skip-record-id-key = true
skip-datetime = true

[[test.results]]
value = "[{ id: user:059nm0eomnrwvq692i3i, updated: d'2025-10-07T07:48:06.855302Z' }]"
skip-record-id-key = true
skip-datetime = true

*/

DEFINE FIELD updated ON TABLE user VALUE time::now();

-- Ignores 1900 date, sets `updated` to current time
CREATE user SET updated = d"1900-01-01";
-- Ignores again, updates to current time
UPDATE user SET updated = d"1900-01-01";
