/**[test]

[[test.results]]
value = "[{ id: travel:yellowstone, routes: { type: 'MultiLineString', coordinates: [[[10, 11.2], [10.5, 11.9]], [[11, 12.2], [11.5, 12.9], [12, 13]]] } }]"

*/

CREATE travel:yellowstone SET routes = {
	type: "MultiLineString",
	coordinates: [
		[ [10.0, 11.2], [10.5, 11.9] ],
		[ [11.0, 12.2], [11.5, 12.9], [12.0, 13.0] ]
	]
}
