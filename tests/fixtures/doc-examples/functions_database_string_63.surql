/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ of: 'SurrealDB	surrealdb', score: 187 }, { of: 'SurrealDB	SurralDB', score: 165 }, { of: 'SurrealDB	Surreal', score: 151 }, { of: 'SurrealDB	real', score: 75 }, { of: 'SurrealDB	eal', score: 55 }, { of: 'SurrealDB	DB', score: 41 }]"

*/

LET $original = "SurrealDB";
LET $strings = ["SurralDB", "surrealdb", "DB", "Surreal", "real", "basebase", "eel", "eal"];

FOR $string IN $strings {
    LET $score = string::similarity::fuzzy($original, $string);
    IF $score > 0 {
        CREATE comparison SET of = $original + '\t' + $string,
        score = $score
    };
};

SELECT of, score FROM comparison ORDER BY score DESC;
