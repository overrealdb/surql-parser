/**[test]

[[test.results]]
value = "[{ id: time_data:[d'2024-07-23T00:00:00Z'] }]"

[[test.results]]
value = "[{ id: time_data:[d'2024-07-24T00:00:00Z'] }]"

[[test.results]]
value = "[{ id: time_data:[d'2024-07-25T00:00:00Z'] }]"

[[test.results]]
value = "[{ id: time_data:[d'2024-07-24T00:00:00Z'] }]"

[[test.results]]
value = "[{ id: time_data:[d'2024-07-24T00:00:00Z'] }, { id: time_data:[d'2024-07-25T00:00:00Z'] }]"

[[test.results]]
value = "[{ id: time_data:[d'2024-07-23T00:00:00Z'] }, { id: time_data:[d'2024-07-24T00:00:00Z'] }, { id: time_data:[d'2024-07-25T00:00:00Z'] }]"

*/

CREATE time_data:[d'2024-07-23T00:00:00.000Z'];
CREATE time_data:[d'2024-07-24T00:00:00.000Z'];
CREATE time_data:[d'2024-07-25T00:00:00.000Z'];
-- Records from the 24th to the 25th
SELECT * FROM time_data:[d'2024-07-24']..[d'2024-07-25'];
-- Records from the 24th
SELECT * FROM time_data:[d'2024-07-24']..;
-- All records
SELECT * FROM time_data:[NONE]..;
