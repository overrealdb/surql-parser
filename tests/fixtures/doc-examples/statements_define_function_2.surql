/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ optional_present: true, required_present: true }"

[[test.results]]
value = "{ optional_present: false, required_present: true }"

*/

DEFINE FUNCTION fn::last_option($required: number, $optional: option<number>) {
	RETURN {
		required_present: type::is_number($required),
		optional_present: type::is_number($optional),
	}
};

RETURN fn::last_option(1, 2);
-- { required_present: true, optional_present: true }

RETURN fn::last_option(1);
-- { required_present: true, optional_present: false };
