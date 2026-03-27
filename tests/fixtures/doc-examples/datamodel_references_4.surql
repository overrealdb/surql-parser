/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: comic_book:one, owned_by: [], title: 'Loki, God of Stories' }]"

[[test.results]]
value = "[{ comics: [comic_book:one], id: person:one }]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ comics: [comic_book:one], id: person:one }]"

*/

DEFINE FIELD comics ON person TYPE option<array<record<comic_book>>> REFERENCE;
DEFINE FIELD owned_by ON comic_book COMPUTED <~person;

CREATE comic_book:one SET title = "Loki, God of Stories";
CREATE person:one SET comics = [comic_book:one];
DELETE comic_book:one;
SELECT * FROM person;
