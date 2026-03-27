/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[]"

*/

-- Define a function that fetches all expiration years from the payment_details table and denies access to all none-admin users
DEFINE FUNCTION fn::fetchAllPaymentDetails() {
	SELECT stored_cards.expiry_year FROM payment_details LIMIT 5
} PERMISSIONS NONE;

RETURN fn::fetchAllPaymentDetails();
