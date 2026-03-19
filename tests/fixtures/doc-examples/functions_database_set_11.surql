/**
[test]

[[test.results]]
value = "5"

[[test.results]]
value = "{ intelligence: 15, name: 'Mardine', strength: 10 }"

*/

{1, 2, 5}.find(|$num| $num >= 3);
-- 5

{
	{ strength: 15, intelligence: 6, name: 'Dom the Magnificent' },
	{ strength: 10, intelligence: 15, name: 'Mardine' },
	{ strength: 20, intelligence: 3, name: 'Gub gub' },
	{ strength: 10, intelligence: 18, name: 'Lumin695' }
}.find(|$c| $c.strength > 9 AND $c.intelligence > 9);
-- { intelligence: 15, name: 'Mardine', strength: 10 }
