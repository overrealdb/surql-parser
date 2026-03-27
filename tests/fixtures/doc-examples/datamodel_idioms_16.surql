/**[test]

[[test.results]]
value = "[{ id: planet:unknown_planet }]"

[[test.results]]
value = "[{ id: city:el_dorado, name: 'El Dorado' }]"

[[test.results]]
value = "[{ id: explorer:drake, name: 'Drake' }]"

[[test.results]]
value = "[{ id: explorer:local_guide, name: 'Local Guide' }]"

[[test.results]]
value = "[{ id: discovered:4y6p19onn8a0zlgf80rg, in: explorer:drake, out: planet:unknown_planet }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: visited:lnjo8yonp28501y75slq, in: explorer:drake, out: city:el_dorado }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: assisted:5cr7hphseo7t2m87yhwd, in: explorer:local_guide, out: explorer:drake }]"
skip-record-id-key = true

*/

CREATE planet:unknown_planet;
CREATE city:el_dorado          SET name = "El Dorado";
CREATE explorer:drake          SET name = "Drake";
CREATE explorer:local_guide    SET name = "Local Guide";

RELATE explorer:drake->discovered->planet:unknown_planet;
RELATE explorer:drake->visited->city:el_dorado;
RELATE explorer:local_guide->assisted->explorer:drake;
