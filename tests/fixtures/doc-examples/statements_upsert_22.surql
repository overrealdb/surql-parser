/**[test]

[[test.results]]
value = "[{ id: person:8v3or7b2uqfnscvptzkd, settings: { marketing: true } }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: person:tobie, settings: { marketing: true } }]"

*/

-- Inserts a new record with a single field and random ID
UPSERT person MERGE {
	settings: {
		marketing: true,
	},
};

-- Updates certain fields on a specific record
UPSERT person:tobie MERGE {
	settings: {
		marketing: true,
	},
};
