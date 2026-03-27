/**[test]

[[test.results]]
value = "[{ bedrooms: 5, floors: 2, id: house:['Dalhurst Way', 218] }]"

[[test.results]]
value = "[{ bedrooms: 12, floors: 4, id: house:['Middle of nowhere', 0] }]"

*/

CREATE house:["Dalhurst Way", 218] SET floors = 2, bedrooms = 5;
CREATE house:["Middle of nowhere", 0] SET floors = 4, bedrooms = 12;
