/**[test]

[[test.results]]
value = "NONE"

*/

-- Define a function that checks if a relation exists between two nodes
DEFINE FUNCTION fn::relation_exists(
	$in: record,
	$tb: string,
	$out: record
) {
	-- Check if a relation exists between the two nodes.
	LET $results = SELECT VALUE id FROM type::table($tb) WHERE in = $in AND out = $out;
	-- Return true if a relation exists, false otherwise
    RETURN array::len($results) > 0;
};
