/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: event:w2lhv58f7c9z7xo4nqkq, time: d'2025-07-03T07:18:52.841140Z' }]"
skip-record-id-key = true

*/

DEFINE FIELD time ON event TYPE datetime;
// highlight-next-line
CREATE event SET time = d"2025-07-03T07:18:52.84114Z";
