/**[test]

[[test.results]]
value = "51"

[[test.results]]
value = "2997"

[[test.results]]
value = "151"

*/

-- returns 51
RETURN string::similarity::fuzzy("DB", "DB");
-- returns 2997
RETURN string::similarity::fuzzy(
  "SurrealDB Cloud Beta is now live! We are excited to announce that we are inviting users from the waitlist to join. Stay tuned for your invitation!", "SurrealDB Cloud Beta is now live! We are excited to announce that we are inviting users from the waitlist to join. Stay tuned for your invitation!"
);
-- returns 151 despite nowhere close to exact match
RETURN string::similarity::fuzzy(
  "SurrealDB Cloud Beta is now live! We are excited to announce that we are inviting users from the waitlist to join. Stay tuned for your invitation!", "Surreal"
);
