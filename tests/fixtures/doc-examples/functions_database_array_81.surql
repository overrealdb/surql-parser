/**[test]

[[test.results]]
value = "{ money: 11650, name: 'Daughter and Father and Grandfather and Great-grandmother' }"

*/

[
    {
        name: "Daughter",
        money: 100
    },
    {
        name: "Father",
        money: 1000
    },
    {
        name: "Grandfather",
        money: 550
    },
    {
        name: "Great-grandmother",
        money: 10000
    }
].reduce(|$one, $two, $index| IF $index > 2 { $one } ELSE {
    {
        name: $one.name + " and " + $two.name,
        money: $one.money + $two.money
    }
});
