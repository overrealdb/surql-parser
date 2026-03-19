/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[]"

*/

DEFINE TABLE review DROP;
-- Define a table as a view which aggregates data from the review table
DEFINE TABLE avg_product_review TYPE NORMAL AS
SELECT
	count() AS number_of_reviews,
	math::mean(<float> rating) AS avg_review,
	->product.id AS product_id,
	->product.name AS product_name
FROM review
GROUP BY product_id, product_name;

-- Query the projection
SELECT * FROM avg_product_review;
