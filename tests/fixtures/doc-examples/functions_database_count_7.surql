/**[test]

[[test.results]]
value = "[{ id: user:dbb4iogxntigq3m11rdp }]"

[[test.results]]
value = "[{ count: 1 }]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ count: 100001 }"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ count: 100001 }]"

*/

CREATE user;
-- One record in table, very fast
SELECT count() FROM user GROUP ALL;

-- 10,000 new records,
-- count() takes a bit longer than before
CREATE |user:10000| RETURN NONE;
SELECT count() FROM user GROUP ALL;

-- Add index, wait a moment for it to build
DEFINE INDEX user_count ON user COUNT;
-- count() very performant again
SELECT count() FROM user GROUP ALL;
