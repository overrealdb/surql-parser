/**[test]

[[test.results]]
value = "[{ id: person:tobie, locations: { type: 'MultiPoint', coordinates: [[10, 11.2], [10.5, 11.9]] } }]"

*/

CREATE person:tobie SET locations = {
	type: "MultiPoint",
	coordinates: [
		[10.0, 11.2],
		[10.5, 11.9]
	],
};
