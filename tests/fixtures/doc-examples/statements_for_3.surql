/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

LET $table1 = "person";
LET $table2 = "cat";

FOR $key in 0..4 {
    CREATE type::record($table1, $key);
	  CREATE type::record($table2, $key);
};
