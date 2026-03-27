/**[test]

[[test.results]]
value = "NONE"

*/

-- Set can_vote to true for every person over 18 years old.
FOR $person IN (SELECT VALUE id FROM person WHERE age >= 18) {
	UPDATE $person SET can_vote = true;
};
