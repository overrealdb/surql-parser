/**[test]

[[test.results]]
value = "'Sur�rea�lDB'"

[[test.results]]
value = "'SurrealDB'"

*/

-- Contains some invalid bytes
type::string_lossy(<bytes>[83, 117, 114, 255, 114, 101, 97, 254, 108, 68, 66]);
-- valid bytes
type::string_lossy(<bytes>[ 83, 117, 114, 114, 101, 97, 108, 68, 66 ]);
