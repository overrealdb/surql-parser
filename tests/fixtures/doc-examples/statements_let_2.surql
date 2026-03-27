/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "'integer'"

*/

LET $num = 10;

LET $num_type =
         IF type::is_int($num)     { "integer" }
    ELSE IF type::is_decimal($num) { "decimal" }
    ELSE IF type::is_float($num)   { "float"   };

RETURN $num_type;
-- 'integer'
