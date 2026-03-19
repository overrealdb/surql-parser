/**[test]

[[test.results]]
value = "[{ id: person:aristotle }, { id: article:on_sleep_and_sleeplessness }, { id: article:on_dreams }]"

[[test.results]]
value = "[{ id: wrote:j0i75u90qswob99b779m, in: person:aristotle, out: article:on_sleep_and_sleeplessness, time_written: d'-0330-01-01T00:00:00Z' }, { id: wrote:cyqldu8ma76nh1q9jprq, in: person:aristotle, out: article:on_dreams, time_written: d'-0330-01-01T00:00:00Z' }]"
skip-datetime = true 
skip-record-id-key = true

*/

CREATE 
	person:aristotle,
	article:on_sleep_and_sleeplessness,
	article:on_dreams;
RELATE person:aristotle->wrote->[
		article:on_sleep_and_sleeplessness,
		article:on_dreams
	]
	// Written sometime around the year 330 BC
	SET time_written = d"-0330-01-01";
