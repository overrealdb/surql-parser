/**[test]

[[test.results]]
value = "[{ id: person:grandfather }, { id: person:father }, { id: person:son }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ grandsons: [], id: person:father, sons: [person:son] }, { grandsons: [person:son], id: person:grandfather, sons: [person:father] }, { grandsons: [], id: person:son, sons: [] }]"

*/

CREATE person:grandfather, person:father, person:son;

FOR $pair IN array::windows(["grandfather", "father", "son"], 2) {
    LET $first = type::record("person", $pair[0]);
    LET $second = type::record("person", $pair[1]);
    RELATE $first->father_of->$second;
};

SELECT 
  id, 
  ->father_of->person AS sons,
  ->father_of->person->father_of->person AS grandsons
FROM person;
