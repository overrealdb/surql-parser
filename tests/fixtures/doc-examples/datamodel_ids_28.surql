/**[test]

[[test.results]]
value = "[{ id: event:[user:one, 'debug', d'2025-05-05T08:00:00Z'], info: 'Logged in' }]"

[[test.results]]
value = "[{ id: event:[user:one, 'debug', d'2025-05-05T08:10:00Z'], info: 'Logged out' }]"

[[test.results]]
value = "[{ id: event:[user:two, 'debug', d'2025-05-05T08:01:00Z'], info: 'Logged in' }]"

*/


CREATE event:[user:one, "debug", d'2025-05-05T08:00:00Z'] SET info = "Logged in";
CREATE event:[user:one, "debug", d'2025-05-05T08:10:00Z'] SET info = "Logged out";
CREATE event:[user:two, "debug", d'2025-05-05T08:01:00Z'] SET info = "Logged in";
