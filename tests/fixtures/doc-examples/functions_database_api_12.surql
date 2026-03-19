api::res::body({}, || {}, "json").{
    body: <string>body,
    context,
    headers,
    status
};

-- Returns:
{
	body: 'null',
	context: {},
	headers: {
		"content-type": 'application/json'
	},
	status: 200
};

api::res::body({}, || {}, "jsonnn").{
    body: <string>body,
    context,
    headers,
    status
};

-- Returns:
'Failed to decode BodyStrategy, no variants matched'
