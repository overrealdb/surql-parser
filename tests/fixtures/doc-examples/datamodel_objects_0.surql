/**[test]

[[test.results]]
value = "[{ id: person:ix25bwtvcx5s543eejrk, metadata: { activities: ['clicked link', 'contact form', 'read email', 'viewed website', 'viewed website', 'viewed website', 'read email'], information: { age: 23, gender: 'm' }, interest_level: 83.67f, marketing: true } }]"
skip-record-id-key = true

*/

CREATE person SET metadata = {
	interest_level: 83.67,
	information: {
		age: 23,
		gender: 'm',
	},
	marketing: true,
	activities: [
		"clicked link",
		"contact form",
		"read email",
		"viewed website",
		"viewed website",
		"viewed website",
		"read email",
	]
};
