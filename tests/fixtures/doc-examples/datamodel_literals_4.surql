/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ error_info: { error: 'Deprecated', message: "You shouldn't use this anymore" }, id: information:obqulsqif466kp1t3agq }]"
skip-record-id-key = true

[[test.results]]
error = ""Couldn't coerce value for field `error_info` of `information:qbohn4wu4l2t81wj2fb3`: Expected `{ error: 'Continue' } | { error: 'RetryWithId', id: string } | { error: 'Deprecated', message: string }` but found `\"You shouldn't use this anymore\"`""

*/

DEFINE FIELD error_info ON TABLE information TYPE
      { error: "Continue" }
    | { error: "RetryWithId", id: string }
    | { error: "Deprecated", message: string };

CREATE information SET
	error_info = { error: "Deprecated", message: "You shouldn't use this anymore" };
-- Doesn't conform to definition, will not work
CREATE information SET
	error_info = "You shouldn't use this anymore";
