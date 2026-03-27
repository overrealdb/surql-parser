/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: wrote:9zyb1fytya4cykc47j0o, in: person:l19zjikkw1p1h9o6ixrg, out: article:8nkk6uj4yprt49z7y3zm, time: { written: d'2025-10-08T04:28:53.000469Z' } }]"
skip-record-id-key = true

*/

LET $time = time::now();
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    CONTENT {
        time: {
            written: $time
        }
    };
