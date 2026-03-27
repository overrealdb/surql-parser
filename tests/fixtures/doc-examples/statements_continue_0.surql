/**[test]

[[test.results]]
value = "NONE"

*/

-- Set can_vote to true for every person over 18 years old.
FOR $person IN (SELECT id, age FROM person) {
	IF ($person.age < 18) {
		CONTINUE;
	};

	UPDATE $person.id SET can_vote = true;
};
