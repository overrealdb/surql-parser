-- Update all records with the same content
UPDATE person CONTENT {
	name: 'John',
	company: 'SurrealDB',
	skills: ['Rust', 'Go', 'JavaScript'],
};

-- Oops, now they are both named John.
-- Update a specific record with some content
UPDATE person:tobie CONTENT {
	name: 'Tobie',
	company: 'SurrealDB',
	skills: ['Rust', 'Go', 'JavaScript'],
};
