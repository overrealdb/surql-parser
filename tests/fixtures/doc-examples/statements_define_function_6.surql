/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: person:1 }, { id: person:2 }, { id: person:3 }, { id: person:4 }, { id: person:5 }, { id: person:6 }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ "->to": { "->?": [person:3, person:4, person:6, person:5, person:2] }, id: person:1 }, { "->to": { "->?": [person:6, person:3, person:5, person:4] }, id: person:2 }, { "->to": { "->?": [person:6, person:5, person:4] }, id: person:3 }, { "->to": { "->?": [person:5, person:6] }, id: person:4 }, { "->to": { "->?": [person:6] }, id: person:5 }, { "->to": { "->?": [] }, id: person:6 }]"

*/

DEFINE FUNCTION fn::relate_all($records: array<record>) {
  IF $records.len() < 2 {
      -- Don't do anything, ending the recursion
  }  ELSE {
      LET $first = $records[0];
      LET $remainder = $records[1..];
      FOR $counterpart IN $remainder {
          RELATE $first->to->$counterpart;
      };
      fn::relate_all($remainder);
  }
};

CREATE |person:1..8|;

fn::relate_all(SELECT VALUE id FROM person);

SELECT id, ->to->? FROM person;
