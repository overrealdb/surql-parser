/**[test]

[[test.results]]
error = "'Cannot use the `VALUE` keyword on the `id` field.'"

*/

DEFINE FIELD id ON user VALUE rand::int(1, 1000000000) READONLY;
