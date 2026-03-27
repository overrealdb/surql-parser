/**[test]

[[test.results]]
value = "['person:one', 'user:two', "['user:three', 'user:four']", 'not_a_person_or_user']"

*/

<array<record<user | person> | array<record<user>> | string>> [
	'person:one',
	'user:two',
	[
		'user:three',
		'user:four'
	],
	'not_a_person_or_user'
];
