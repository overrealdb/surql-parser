/**[test]

[[test.results]]
value = "[{ click_count: 1, id: webpage:home }]"

*/

-- UPSERT a document and increment a numeric value
UPSERT webpage:home SET click_count += 1;
