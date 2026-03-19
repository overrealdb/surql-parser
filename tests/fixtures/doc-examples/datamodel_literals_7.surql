LET $info = INSERT INTO information [
	{ error_info: { Continue: { message: "" } }},
	{ error_info: { Retry: { error: "Retrying", after: 1s } }},
	{ error_info: { Deprecated: { message: "Thought I said you shouldn't use this anymore" } }}
];

fn::handle_error($info[0].id);
fn::handle_error($info[1].id);
fn::handle_error($info[2].id);
