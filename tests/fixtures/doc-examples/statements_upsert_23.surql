/**[test]

[[test.results]]
value = "[{ Engineering: 'true', id: person:tobie }]"

*/

-- Patch the JSON response
UPSERT person:tobie PATCH [
	{
		"op": "add",
		"path": "Engineering",
		"value": "true"
	}
];
