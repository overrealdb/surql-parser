/**[test]

[[test.results]]
value = "[{ circumference: 10, id: circle:uvw3g4dli4x77xejcjej }]"
skip-record-id-key = true

[[test.results]]
value = "[{ circumference: 10, id: circle:uvw3g4dli4x77xejcjej, radius: 15.707963267948966f }]"
skip-record-id-key = true

*/

CREATE circle SET circumference = 10;
UPDATE circle SET radius = circumference / ( 2 * MATH::PI );
