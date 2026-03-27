/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ data_for: user:one, id: user_data:24gujsffwl4vt2zgp2yl, some: 'data' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ data_for: user:one, id: user_data:24gujsffwl4vt2zgp2yl, last_edited: d'2025-10-08T03:22:23.325474Z', some: 'data', times_updated: 1 }]"
skip-record-id-key = true

*/

DEFINE FIELD data_for ON user_data TYPE record<user>;
DEFINE INDEX one_user ON user_data FIELDS data_for UNIQUE;

INSERT INTO user_data {
    data_for: user:one,
    some: "data"
} ON DUPLICATE KEY UPDATE times_updated += 1, last_edited = time::now();

INSERT INTO user_data {
    data_for: user:one,
    some_more: "data"
} ON DUPLICATE KEY UPDATE times_updated += 1, last_edited = time::now();
