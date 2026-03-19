/**[test]

[[test.results]]
value = "[{ buildings: { type: 'GeometryCollection', geometries: [{ type: 'MultiPoint', coordinates: [[10, 11.2], [10.5, 11.9]] }, { type: 'Polygon', coordinates: [[[-0.38314819, 51.37692386], [0.1785278, 51.37692386], [0.1785278, 51.6146057], [-0.38314819, 51.6146057], [-0.38314819, 51.37692386]]] }, { type: 'MultiPolygon', coordinates: [[[[10, 11.2], [10.5, 11.9], [10.8, 12], [10, 11.2]]], [[[9, 11.2], [10.5, 11.9], [10.3, 13], [9, 11.2]]]] }] }, id: university:oxford }]"

*/

CREATE university:oxford SET buildings = {
	type: "GeometryCollection",
	geometries: [
		{
			type: "MultiPoint",
			coordinates: [
				[10.0, 11.2],
				[10.5, 11.9]
			],
		},
		{
			type: "Polygon",
			coordinates: [[
				[-0.38314819, 51.37692386], [0.1785278, 51.37692386],
				[0.1785278, 51.61460570], [-0.38314819, 51.61460570],
				[-0.38314819, 51.37692386]
			]]
		},
		{
			type: "MultiPolygon",
			coordinates: [
				[
					[ [10.0, 11.2], [10.5, 11.9], [10.8, 12.0], [10.0, 11.2] ]
				],
				[
					[ [9.0, 11.2], [10.5, 11.9], [10.3, 13.0], [9.0, 11.2] ]
				]
			]
		}
	]
};
