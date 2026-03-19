/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ city: 'London', id: weather:⟨2025-10-03T01:13:14.238633Z⟩ }]"
skip-record-id-key = true

*/

LET $now = time::now();

CREATE type::record("weather", $now) SET city = 'London';
