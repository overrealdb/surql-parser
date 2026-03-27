/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[person:e71cnhnwdb6yuz8is8kq]"
skip-record-id-key = true

[[test.results]]
value = "[{ firstname: 'Thanos', id: person:e71cnhnwdb6yuz8is8kq, lastname: 'Johnson' }]"
skip-record-id-key = true

*/

DEFINE FUNCTION fn::person::create($firstname: string, $lastname: string) {
	LET $person = CREATE person CONTENT {
		firstname: $firstname,
		lastname: $lastname,
	};

	-- The RETURN statement will set the return value of the custom function, and further queries will not be executed.
	RETURN $person.id;

    -- This query will never be executed
    CREATE person SET firstname = "Stephen", lastname = "Strange";
};

fn::person::create("Thanos", "Johnson");
SELECT * FROM person;
