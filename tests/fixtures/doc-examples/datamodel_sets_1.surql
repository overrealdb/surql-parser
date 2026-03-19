/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

*/
{,}.is_set();  -- true
{9,}.is_set(); -- true

{}.is_set();   -- false
{9}.is_set();  -- false
