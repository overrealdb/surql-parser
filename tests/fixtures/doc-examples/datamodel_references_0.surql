/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE FIELD comics ON person TYPE option<array<record<comic_book>>> REFERENCE;
-- Also works as `option` desugars to this syntax
DEFINE FIELD comics ON person TYPE array<record<comic_book>> | NONE REFERENCE;

-- `comics` field might not be a record, does not work
DEFINE FIELD comics ON person TYPE array<record<comic_book>> | string REFERENCE;
-- Not top-level field, does not work
DEFINE FIELD metadata.comics ON person TYPE array<record<comic_book>> REFERENCE;
