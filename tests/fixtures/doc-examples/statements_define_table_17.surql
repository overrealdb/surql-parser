/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = ""The record 'city:one' does not exist""

*/

DEFINE TABLE road_to TYPE RELATION IN city OUT city ENFORCED;

RELATE city:one->road_to->city:three SET
    distance = 5.5,
    slope = 30.0;
