DEFINE TABLE person SCHEMALESS;
DEFINE TABLE person_stats AS
	SELECT
		count(),
		age,
		math::stddev(score) AS score_stddev,
		math::variance(score) AS score_variance
	FROM person
	GROUP BY age;

INSERT INTO person [
    { id: person:alice,          age: 25, score: 80 },
    { id: person:alices_rival,   age: 25, score: 88 },
    { id: person:bob,            age: 24, score: 90 },
    { id: person:bobs_rival,     age: 24, score: 99 },
    { id: person:charlie,        age: 23, score: 70 },
    { id: person:charlies_rival, age: 23, score: 77 }
];

SELECT * FROM person_stats WHERE age >= 24;
