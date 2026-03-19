/**[test]

[[test.results]]
value = "[{ distance: 12.4f, id: road_to:jd2f9pjzecb0ihmo2mbp, in: city:one, out: city:two, slope: 5.4f }]"
skip-record-id-key = true

*/

RELATE city:one->road_to->city:two SET
    distance = 12.4,
    slope = 5.4;
