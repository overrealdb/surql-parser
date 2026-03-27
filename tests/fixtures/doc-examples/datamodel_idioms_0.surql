/**[test]

[[test.results]]
value = "[{ address: { city: 'New York', country: 'USA' }, age: 30, id: person:oakmq2g0njddr5ysmils, name: 'John Doe' }]"
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
