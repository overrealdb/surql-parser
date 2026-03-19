/**[test]

[[test.results]]
value = "[{ id: university:oxford, locations: { type: 'MultiPolygon', coordinates: [[[[10, 11.2], [10.5, 11.9], [10.8, 12], [10, 11.2]]], [[[9, 11.2], [10.5, 11.9], [10.3, 13], [9, 11.2]]]] } }]"

*/

CREATE university:oxford SET locations = {
	type: "MultiPolygon",
	coordinates: [
		[
			[ [10.0, 11.2], [10.5, 11.9], [10.8, 12.0], [10.0, 11.2] ]
		],
		[
			[ [9.0, 11.2], [10.5, 11.9], [10.3, 13.0], [9.0, 11.2] ]
		]
	]
};
