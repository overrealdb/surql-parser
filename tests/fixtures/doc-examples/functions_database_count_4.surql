/**[test]

[[test.results]]
value = "[{ count: 3 }]"

*/

SELECT 
	count() 
FROM [
	{ age: 33 }, 
	{ age: 45 }, 
	{ age: 39 }
] 
GROUP ALL;
