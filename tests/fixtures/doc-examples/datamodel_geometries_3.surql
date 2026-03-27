/**[test]

[[test.results]]
value = "[{ distance: { type: 'LineString', coordinates: [[-0.118092, 51.509865], [0.1785278, 51.37692386]] }, id: city:london }]"

*/

CREATE city:london SET distance = {
    type: "LineString",
    coordinates: [[-0.118092, 51.509865],[0.1785278, 51.37692386]],
};
