SELECT id, title, search::score(1) AS score FROM book
	WHERE title @1@ 'rust web'
	ORDER BY score DESC;
