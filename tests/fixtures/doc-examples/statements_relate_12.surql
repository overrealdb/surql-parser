/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: wrote:o6kz4fqcqgqok1xqoqm0, in: person:l19zjikkw1p1h9o6ixrg, out: article:8nkk6uj4yprt49z7y3zm, time: { written: d'2025-10-08T04:29:36.089521Z' } }]"
skip-record-id-key = true

*/

LET $time = time::now();

RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = $time;
