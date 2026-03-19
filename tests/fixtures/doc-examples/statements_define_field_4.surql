/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Set a field to have the string data type
DEFINE FIELD email ON TABLE user TYPE string;

-- Set a field to have the datetime data type
DEFINE FIELD created ON TABLE user TYPE datetime;

-- Set a field to have the bool data type
DEFINE FIELD locked ON TABLE user TYPE bool;

-- Set a field to have the number data type
DEFINE FIELD login_attempts ON TABLE user TYPE number;
