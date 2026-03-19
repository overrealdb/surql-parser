LET $now = time::now();
[$now, time::from_ulid(rand::ulid($now))];

-- Output:
[
	d'2026-01-29T02:07:06.494218Z',
	d'2026-01-29T02:07:06.494Z'
]
