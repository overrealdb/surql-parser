SELECT id, title, search::offsets(1) AS title_offsets
	FROM book WHERE title @1@ 'rust web';
