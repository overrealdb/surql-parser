/**[test]

[[test.results]]
value = "[{ id: person:jaime, name: 'Jaime', surname: 'Morgan Hitchcock' }, { id: person:tobie, name: 'Tobie', surname: 'Morgan Hitchcock' }]"

*/

INSERT IGNORE INTO person [
   { id: "jaime", name: "Jaime", surname: "Morgan Hitchcock" },
   { id: "tobie", name: "Tobie", surname: "Morgan Hitchcock" },

   { id: "jaime", name: "Jaime", surname: "Morgan Hitchcock" }, -- will not throw an error
];
