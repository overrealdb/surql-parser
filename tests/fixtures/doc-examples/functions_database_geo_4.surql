/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "8268604.251890703f"

*/
let $london = (-0.04592553673505285, 51.555282574465764);
let $harare = (30.463880214538577, -17.865161568822085);
RETURN geo::distance($london, $harare);
