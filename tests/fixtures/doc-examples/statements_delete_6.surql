/**[test]

[[test.results]]
value = "[{ age: 4, id: cat:one }]"

[[test.results]]
value = "[{ id: cat:two }]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ id: cat:two }]"

*/

CREATE cat:one SET age = 4;
CREATE cat:two;
DELETE cat WHERE age;
SELECT * FROM cat;
