/**[test]

[[test.results]]
value = "[{ age: 21, id: person:1, name: 'John', obj: { a: 1, b: 2, c: { d: 3, e: 4, f: 5 } } }]"

*/


CREATE person:1 SET name = 'John', age = 21, obj = { a: 1, b: 2, c: { d: 3, e: 4, f: 5 } };
