/**[test]

[[test.results]]
value = "[{ id: cat:kitten2 }]"

[[test.results]]
value = "[{ id: parent_of:bsgtxiltlv2pd9hf5eqz, in: cat:mr_meow, out: cat:kitten }, { id: parent_of:cr7chl0lot6g4gfj84fe, in: cat:mr_meow, out: cat:kitten2 }, { id: parent_of:y3dtskf9sme2y0hmumlu, in: cat:mrs_meow, out: cat:kitten }, { id: parent_of:lc39ib0lbzw0x1azkee9, in: cat:mrs_meow, out: cat:kitten2 }]"
skip-record-id-key = true

*/

CREATE cat:kitten2;
RELATE [cat:mr_meow, cat:mrs_meow]->parent_of->[cat:kitten, cat:kitten2];
