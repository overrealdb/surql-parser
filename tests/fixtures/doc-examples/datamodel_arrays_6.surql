/**[test]

[[test.results]]
value = "[{ first_mayor: 'Papa Smurf', name: 'Smurfville', population: 55 }]"

*/

[
    {
        name: "Boston",
        population: NONE,
        first_mayor: "John Phillips"
    },
    {
        name: "Smurfville",
        population: 55,
        first_mayor: "Papa Smurf"
    },
    {
        name: "Harrisburg",
        population: 50183,
        first_mayor: NONE
    }
][WHERE $this.population]
 [WHERE $this.first_mayor];
