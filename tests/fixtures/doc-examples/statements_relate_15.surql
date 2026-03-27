/**[test]

[[test.results]]
value = "[]"
skip-record-id-key = true
skip-datetime = true

[[test.results]]
value = "[{ op: 'replace', path: '', value: { id: wrote:me0vergol3jkvn7me15x, in: person:l19zjikkw1p1h9o6ixrg, out: article:8nkk6uj4yprt49z7y3zm, time: { written: d'2025-10-08T04:30:34.214973Z' } } }]"
skip-record-id-key = true
skip-datetime = true

[[test.results]]
value = "NONE"
skip-record-id-key = true
skip-datetime = true

[[test.results]]
value = "[{ id: wrote:pwatbr0s0f2ddwcwqls7, in: person:l19zjikkw1p1h9o6ixrg, out: article:8nkk6uj4yprt49z7y3zm, time: { written: d'2025-10-08T04:30:34.218213Z' } }]"
skip-record-id-key = true
skip-datetime = true

[[test.results]]
value = "[{ time: { written: d'2025-10-08T04:30:34.219729Z' } }]"
skip-datetime = true

[[test.results]]
value = "[{ written: d'2025-10-08T04:30:34.221383Z' }]"
skip-datetime = true

*/

-- Don't return any result
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = time::now()
    RETURN NONE;

-- Return the changeset diff
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = time::now()
    RETURN DIFF;

-- Return the record before changes were applied
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = time::now()
    RETURN BEFORE;

-- Return the record after changes were applied (the default)
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = time::now()
    RETURN AFTER;

-- Return a specific field only from the updated records
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = time::now()
    RETURN time;

-- Return only the value of a specific field without the field name
RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET time.written = time::now()
    RETURN VALUE time;
