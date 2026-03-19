/**[test]

[[test.results]]
value = "{ id: user:3lgc83vechgblizli263, my_name: 'name' }"
skip-record-id-key = true

[[test.results]]
value = "{ id: user:nyzy7aeyup2ygt33ci95, "mi_nómine😊": 'name' }"
skip-record-id-key = true

*/

CREATE ONLY user SET my_name = 'name';
CREATE ONLY user SET `mi_nómine😊` = 'name';
