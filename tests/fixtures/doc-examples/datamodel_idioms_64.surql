/**[test]

[[test.results]]
value = "[{ friends: [{ age: 25, id: 'person:6', name: 'Frank' }, { age: 19, id: 'person:7', name: 'Grace' }, { age: 17, id: 'person:8', name: 'Heidi' }], id: person:5, name: 'Eve' }]"

*/

CREATE person:5 CONTENT {
    name: "Eve",
    friends: [
        {
            id: "person:6",
            name: "Frank",
            age: 25
        },
        {
            id: "person:7",
            name: "Grace",
            age: 19
        },
        {
            id: "person:8",
            name: "Heidi",
            age: 17
        }
    ]
};
