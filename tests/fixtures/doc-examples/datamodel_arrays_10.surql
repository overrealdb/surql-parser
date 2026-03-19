/**[test]

[[test.results]]
value = "[{ coordinates: (98, 65.7), item: 0, name: 'Some city' }, { coordinates: (0, 0.1), item: 1, name: 'Other city' }]"

*/

[
	NONE,
	{
		at: (98, 65.7),
		name: "Some city"
	},
	{
		at: (-190.7, 0),
		name: NONE
	},
    {
        name: "Other city",
        at: (0.0, 0.1)
    },
	{
        name: "Nonexistent city",
        at: (200.0, 66.5)
    }
]
    .filter(|$v| $v != NONE AND $v.name != NONE)
    .filter(|$v| $v.at.is_valid())
    .map(|$v, $i| {
        item: $i,
        name: $v.name,
        coordinates: $v.at
    });
