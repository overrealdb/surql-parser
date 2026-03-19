/**[test]

[[test.results]]
value = "[{ id: cat:f148lyh3zygwenur9nck, name: 'Mr. Meow', nicknames: ['Mr. Cuddlebun'] }]"
skip-record-id-key = true

[[test.results]]
value = "[{ after: { id: cat:f148lyh3zygwenur9nck, name: 'Mr. Meow', nicknames: ['Mr. Cuddlebun', 'Snuggles'] }, before: { id: cat:f148lyh3zygwenur9nck, name: 'Mr. Meow', nicknames: ['Mr. Cuddlebun'] } }]"
skip-record-id-key = true

*/

CREATE cat SET name = "Mr. Meow", nicknames = ["Mr. Cuddlebun"];
UPDATE cat SET nicknames += "Snuggles" WHERE name = "Mr. Meow" RETURN $before, $after;
