/**[test]

[[test.results]]
value = "{ name: 'Venus', orbital_period: 1y31w1d22h, radius: 6051.8f }"

[[test.results]]
value = "{ name: 'Venus', orbital_period: 1y31w1d22h, radius: 6051.8f }"

*/

{ name: "Venus", radius: 6000 } + { radius: 6051.8, orbital_period: 1y31w1d22h };
{ name: "Venus", radius: 6000 }.extend({ radius: 6051.8, orbital_period: 1y31w1d22h });
