/**[test]

[[test.results]]
value = "[{ id: person:one, name: 'Aeon' }]"

[[test.results]]
value = "{ accessed_at: d'2025-10-02T06:18:26.751462Z', name: 'Aeon', name_length: 4 }"
skip-datetime = true

*/

CREATE person:one SET name = "Aeon";

person:one.{
    name,
	-- worked because 'name' can be accessed
    name_length: name.len(),
	-- an expression: did not work before, works now
    accessed_at: time::now()
};
