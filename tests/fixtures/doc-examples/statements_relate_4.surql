/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE INDEX unique_relationships
    ON TABLE wrote
    COLUMNS in, out UNIQUE;
