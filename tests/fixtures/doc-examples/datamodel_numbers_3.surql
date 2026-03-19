/**[test]

[[test.results]]
value = "3.888888888888889dec"

[[test.results]]
value = "3.8888888888888888dec"

*/

-- Creates the imprecise float 3.888888888888889 and casts it into a decimal as 3.888888888888889dec
RETURN <decimal>3.8888888888888888;
-- Uses the input 3.8888888888888888 to directly create a decimal
RETURN 3.8888888888888888dec;
