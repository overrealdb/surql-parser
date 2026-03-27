/**[test]

[[test.results]]
value = "[{ horizon: 34dec, id: event:8vpjsysxnskyfuh9ve87, temperature: 46.5f, year: 2022 }]"
skip-record-id-key = true

*/

CREATE event SET
	year = <int> 2022,
	temperature = <float> 41.5 + 5f,
	horizon = <decimal> 31 + 3dec
;
