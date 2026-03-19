/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = "'An error occurred: Tried to make a { id: data:one, num: 11 } but `num` field requires an even number'"

*/

DEFINE FIELD num ON data TYPE int ASSERT {
    IF $input % 2 = 0 {
        RETURN true
    } ELSE {
        THROW "Tried to make a " + <string>$this + " but `num` field requires an even number"
    }
};

CREATE data:one SET num = 11;
