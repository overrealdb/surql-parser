SELECT id, search::highlight('<b>', '</b>', 1) AS title
	FROM book WHERE title @1@ 'rust web';
