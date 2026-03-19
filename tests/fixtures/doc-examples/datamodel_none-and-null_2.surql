/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

DEFINE FUNCTION fn::do_stuff() -> NONE {
  // Code that should return nothing
};

DEFINE FIELD middle_name ON TABLE user TYPE string | NONE; // Equivalent to option<string>

DEFINE FIELD value ON temperature TYPE float | decimal | NONE; // Equivalent to option<float|decimal>
