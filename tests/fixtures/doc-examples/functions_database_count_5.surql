/**[test]

[[test.results]]
value = "[{ count: 2 }]"

*/

SELECT 
	count(age > 35) 
FROM [
	{ age: 33 }, 
	{ age: 45 }, 
	{ age: 39 }
] 
GROUP ALL;
