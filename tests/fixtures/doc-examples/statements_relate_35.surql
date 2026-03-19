/**[test]

[[test.results]]
value = "[{ born: d'1891-01-03T00:00:00Z', id: person:j_r_r_tolkien, name: 'J.R.R. Tolkien' }]"

[[test.results]]
value = "[{ born: '-0428-06-01', id: person:plato, name: 'Plato' }]"

[[test.results]]
value = "[{ id: book:fotr, name: 'The Fellowship of the Ring' }]"

[[test.results]]
value = "[{ id: book:republic, name: 'The Republic', original_name: 'Πολιτεία' }]"

[[test.results]]
value = "[{ id: wrote:j6lto05qluv9nivz4ens, in: person:j_r_r_tolkien, out: book:fotr, written_at: 'North Oxford' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: wrote:y9ydvugxwhrpns6qn4s9, in: person:plato, out: book:republic, written_at: 'Athens' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ books_written_in_athens: [], name: 'J.R.R. Tolkien' }, { books_written_in_athens: [{ id: book:republic, name: 'The Republic', original_name: 'Πολιτεία' }], name: 'Plato' }]"

[[test.results]]
value = "[{ books_about_rings: [{ id: book:fotr, name: 'The Fellowship of the Ring' }], name: 'J.R.R. Tolkien' }, { books_about_rings: [], name: 'Plato' }]"

*/

CREATE person:j_r_r_tolkien SET
	name = "J.R.R. Tolkien",
	born = d'1891-01-03';
-- Very approximate date of birth
CREATE person:plato SET 
	name = "Plato", 
	born = "-0428-06-01";

CREATE book:fotr SET 
	name = "The Fellowship of the Ring";
CREATE book:republic SET 
	name = "The Republic",
	original_name = "Πολιτεία";

RELATE person:j_r_r_tolkien->wrote->book:fotr SET written_at = "North Oxford";
RELATE person:plato->wrote->book:republic SET written_at = "Athens";

SELECT 
	name,
	-- Isolate 'wrote' to use WHERE
	->(wrote WHERE written_at = "Athens")->book.* AS books_written_in_athens
FROM person;

SELECT 
	name, 
	-- Isolate 'book' to use WHERE
	->wrote->(book WHERE "Ring" IN name).* AS books_about_rings
FROM person;
