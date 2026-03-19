/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

*/

string::matches("a", "a|b");
string::matches("colour", "col(o|ou)r");
string::matches("COULEUR", "((?i)col(o|ou)r|couleur)");
