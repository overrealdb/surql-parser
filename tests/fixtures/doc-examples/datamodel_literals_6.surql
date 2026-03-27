/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE FUNCTION fn::handle_error($data: record<information>) -> string {
	LET $err = $data.error_info;
	RETURN IF $err.Continue {
		"Continue"
	}
	ELSE IF $err.Retry {
		sleep($err.Retry.after);
		"Now retrying again"
	}
	ELSE IF $err.Deprecated {
		$err.Deprecated.message
	}
};
