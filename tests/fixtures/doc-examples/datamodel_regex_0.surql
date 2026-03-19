/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

*/

-- Either 'a' or 'b'
<regex> "a|b" = "a";

-- Either color or colour
<regex> "col(o|ou)r" = "colour";

-- Case-insensitive match on English color, colour, or French couleur
<regex> "((?i)col(o|ou)r|couleur)" = "COULEUR";
