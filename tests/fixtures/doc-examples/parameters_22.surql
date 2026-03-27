/**[test]

[[test.results]]
value = "NONE"

*/

FOR $table IN ["test_user", "test_client"] {
    DEFINE TABLE $table;
    // Do some tests
    REMOVE TABLE $table;
};
