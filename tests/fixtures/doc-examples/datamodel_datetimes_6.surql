/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = ""Couldn't coerce value for field `time` of `event:qv8qcjf0w9oowekl36w6`: Expected `datetime` but found `'2025-07-03T07:18:52.841147'`""

*/

DEFINE FIELD time ON event TYPE datetime;
// highlight-next-line
CREATE event SET time = "2025-07-03T07:18:52.841147";
