/**[test]

[[test.results]]
value = "{ name: 'Venus', orbital_period: 1y31w1d22h }"

[[test.results]]
value = "{ name: 'Venus' }"

*/
{ name: 'Venus', orbital_period: 1y31w1d22h, radius: 6051.8 }.remove("radius");
{ name: 'Venus', orbital_period: 1y31w1d22h, radius: 6051.8 }.remove(["radius", "orbital_period"]);
