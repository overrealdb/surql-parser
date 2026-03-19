/**[test]

[[test.results]]
value = "[{ id: city:calgary }, { id: city:daejeon }]"

[[test.results]]
value = "[{ id: sister_of:crb0gb01iybgjhmed1ia, in: city:calgary, out: city:daejeon }]"
skip-record-id-key = true

*/

CREATE city:calgary, city:daejeon;
RELATE city:calgary->sister_of->city:daejeon;
