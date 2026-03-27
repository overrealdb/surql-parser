LET $now = time::now();
[$now, time::from_uuid(rand::uuid($now))];

-- Output:
[
	d'2026-01-29T02:12:13.848476Z',
	d'2026-01-29T02:12:13.848Z'
]
