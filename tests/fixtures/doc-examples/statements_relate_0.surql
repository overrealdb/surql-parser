/**[test]

[[test.results]]
value = "[{ id: person:aristotle }, { id: article:on_sleep_and_sleeplessness }]"

[[test.results]]
value = "[{ id: wrote:2m9047thgn9j9oc05iqh, in: person:aristotle, out: article:on_sleep_and_sleeplessness }]"
skip-record-id-key = true

*/

CREATE person:aristotle, article:on_sleep_and_sleeplessness;
RELATE person:aristotle->wrote->article:on_sleep_and_sleeplessness;
