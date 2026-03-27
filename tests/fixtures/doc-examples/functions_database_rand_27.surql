LET $now = time::now();
[$now, time::from_ulid(rand::ulid($now))];

-- Output:
[
	d'2026-01-29T02:14:10.057075Z',
	d'2026-01-29T02:14:10.057Z'
]
