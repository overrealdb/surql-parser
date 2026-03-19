/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ error_info: { continue: { message: 'Continue' } }, id: information:y0wty8grwins5yxyswea }]"
skip-record-id-key = true

[[test.results]]
value = "[{ error_info: { continue: { message: 'Continue' } }, id: information:wbg4q0k3z97jdy7x5sns }]"
skip-record-id-key = true

[[test.results]]
value = "[{ error_info: { deprecated: { message: "We don't use this anymore" } }, id: information:r4pa3c49one2ofd2bcbw }]"
skip-record-id-key = true

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ error_info: { deprecated: { message: "We don't use this anymore" } }, id: information:r4pa3c49one2ofd2bcbw }]"
skip-record-id-key = true

*/

DEFINE FIELD error_info ON TABLE information TYPE
      { continue: { message: "Continue" } }
    | { retry_with_id: { error: string  } }
    | { deprecated: { message: string   } };

CREATE information SET error_info = { continue: { message: "Continue" }};
CREATE information SET error_info = { continue: { message: "Continue" }};
CREATE information SET error_info = { deprecated: { message: "We don't use this anymore" }};

DELETE information WHERE error_info.continue;
SELECT * FROM information;
