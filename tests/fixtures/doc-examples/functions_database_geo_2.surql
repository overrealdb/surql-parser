/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "164.18154786094604f"

*/

-- LET used here for readability
LET $paris = (2.358058597411099, 48.861109346459536);
LET $le_puy_en_velay = (3.883428431947686, 45.04383588468415);
RETURN geo::bearing($paris, $le_puy_en_velay);
RETURN geo::bearing($le_puy_en_velay, $paris);
