/**[test]

[[test.results]]
value = "[{ id: cat:mr_meow }, { id: cat:mrs_meow }, { id: cat:kitten }]"

[[test.results]]
value = "[{ id: parent_of:a3pm72b0xvltb61s5poy, in: cat:mr_meow, out: cat:kitten }, { id: parent_of:7h1wd7vu01o0kwmns9k1, in: cat:mrs_meow, out: cat:kitten }]"
skip-record-id-key = true

*/

CREATE cat:mr_meow, cat:mrs_meow, cat:kitten;
RELATE [cat:mr_meow, cat:mrs_meow]->parent_of->cat:kitten;
