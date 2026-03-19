/**[test]

[[test.results]]
value = "[{ address: { city: 'New York', country: 'USA' }, age: 30, id: person:hfmq0ckvi2u842jqqx9w, name: 'John Doe' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ address: { city: 'New York', country: 'USA' }, age: 30, id: person:hfmq0ckvi2u842jqqx9w, name: 'JOHN DOE' }]"
skip-record-id-key = true

*/

CREATE person CONTENT {
    name: "John Doe",
    age: 30,
    address: {
      city: "New York",
      country: "USA"
    }
};

SELECT *, name.uppercase() FROM person;
