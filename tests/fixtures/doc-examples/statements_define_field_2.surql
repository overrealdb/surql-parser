/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = "'Cannot set field `fd.*` with type `number` as it mismatched with field `fd` with type `{ a: string, b: number }`'"

*/

DEFINE FIELD OVERWRITE fd ON c TYPE { a: string, b: number };
DEFINE FIELD OVERWRITE fd.* ON c TYPE number;
