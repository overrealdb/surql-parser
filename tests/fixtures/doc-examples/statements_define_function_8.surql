/**[test]

[[test.results]]
value = "NONE"

*/

-- Define a function that fetches all products with the condition that only admin users can access it
DEFINE FUNCTION fn::fetchAllProducts() {
	 SELECT * FROM product LIMIT 10
} PERMISSIONS WHERE $auth.admin = true;
