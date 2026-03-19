SELECT id,
		search::highlight('<b>', '</b>', 1) AS title,
		search::score(1) AS score
FROM book
WHERE title @1@ 'rust web'
ORDER BY score DESC;

[
	{
		id: book:1,
		score: 0.9227996468544006f,
		title: '<b>Rust</b> <b>Web</b> Programming'
	}
]
