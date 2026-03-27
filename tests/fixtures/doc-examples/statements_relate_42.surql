/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

DEFINE FIELD key ON TABLE sister_of VALUE <string>array::sort([in, out]);
DEFINE INDEX only_one_sister_city ON TABLE sister_of FIELDS key UNIQUE;
