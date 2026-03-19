/**[test]

[[test.results]]
value = "['9.1', true, '15h']"

[[test.results]]
value = "['9.1', true, '15h']"

*/

-- Surrealist formatted syntax
 <array<bool | string | float>> [
	'9.1',
	'true',
	15h
];

-- Maybe someone's preferred syntax?
<array
        <bool | string | float>
      >
[ '9.1', 'true', 15h ];
