BEGIN TRANSACTION;
DEFINE INDEX unique_name ON TABLE person FIELDS name UNIQUE;

INSERT INTO person [
    { name: 'Agatha Christie', born: d'1890-09-15' },
    { name: 'Billy Billerson', born: d'1979-09-11' },
	-- Pretend there are is 10,000 more objects here
    { name: 'Agatha Christie', born: d'1955-05-15' },
];

THROW "Reached the end";
COMMIT TRANSACTION;
