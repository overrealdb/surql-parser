-- Patch the JSON response
UPDATE person:tobie PATCH [
	{
		"op": "add",
		"path": "Engineering",
		"value": "true"
	}
]
