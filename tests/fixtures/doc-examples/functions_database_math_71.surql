DEFINE TABLE person SCHEMALESS;
DEFINE TABLE person_stats AS
	SELECT
		count(),
		age,
		math::variance(score) AS score_variance
	FROM person
	GROUP BY age;
