/**[test]

[[test.results]]
value = "[{ celsius: 37.5f, id: temperature:f8xh13smeqhbg7h9o1rw, time: d'2025-10-03T01:03:27.756064Z' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ celsius: 37.5f, id: temperature:a5wxgrluu3n0qxrgkfb9, time: d'2025-10-03T01:03:27.756317Z' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ celsius: 37.5f, id: temperature:01K6KSGTGCZ631S0325WPW3KF1, time: d'2025-10-03T01:03:27.756405Z' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ celsius: 37.5f, id: temperature:u'0199a798-6a0c-7f51-8b87-e856013f98ec', time: d'2025-10-03T01:03:27.756501Z' }]"
skip-record-id-key = true

*/


// Generate a random record ID 20 characters in length
// Charset: `abcdefghijklmnopqrstuvwxyz0123456789`
CREATE temperature:rand() SET time = time::now(), celsius = 37.5;
// Identical to the above CREATE statement, because
// :rand() is the default random ID format
CREATE temperature SET time = time::now(), celsius = 37.5;

// Generate a ULID-based record ID
CREATE temperature:ulid() SET time = time::now(), celsius = 37.5;
// Generate a UUIDv7-based record ID
CREATE temperature:uuid() SET time = time::now(), celsius = 37.5;
