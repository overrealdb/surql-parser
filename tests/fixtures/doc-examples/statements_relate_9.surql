/**[test]

[[test.results]]
value = "[{ id: wrote:oapxwx4gdbpfsz1yxhih, in: person:l19zjikkw1p1h9o6ixrg, metadata: { location: 'Tallinn', time_written: d'2025-10-08T04:28:09.169160Z' }, out: article:8nkk6uj4yprt49z7y3zm }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: wrote:fvrxtrn5cj2fnz2gliyw, in: person:l19zjikkw1p1h9o6ixrg, metadata: { location: 'Tallinn', time_written: d'2025-10-08T04:28:09.171256Z' }, out: article:8nkk6uj4yprt49z7y3zm }]"
skip-record-id-key = true

*/

RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
    SET 
		metadata.time_written = time::now(),
		metadata.location = "Tallinn";


RELATE person:l19zjikkw1p1h9o6ixrg->wrote->article:8nkk6uj4yprt49z7y3zm
	CONTENT {
		metadata: {
			time_written: time::now(),
			location: "Tallinn"
		}
	};
