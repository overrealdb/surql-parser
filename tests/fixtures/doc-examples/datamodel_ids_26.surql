/**[test]

[[test.results]]
value = "[{ id: event:[d'2025-05-05T08:00:00Z', user:one, 'debug'], info: 'Logged in' }]"

[[test.results]]
value = "[{ id: event:[d'2025-05-05T08:10:00Z', user:one, 'debug'], info: 'Logged out' }]"

[[test.results]]
value = "[{ id: event:[d'2025-05-05T08:01:00Z', user:two, 'debug'], info: 'Logged in' }]"

*/

CREATE event:[d'2025-05-05T08:00:00Z', user:one, "debug"] SET info = "Logged in";
CREATE event:[d'2025-05-05T08:10:00Z', user:one, "debug"] SET info = "Logged out";
CREATE event:[d'2025-05-05T08:01:00Z', user:two, "debug"] SET info = "Logged in";
