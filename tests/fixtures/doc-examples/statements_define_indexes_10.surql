/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Define the an analyzer with
DEFINE ANALYZER example_ascii TOKENIZERS class FILTERS ascii;
-- Since 3.0.0-beta: only FULLTEXT used to benefit from concurrent full-text search
DEFINE INDEX userNameIndex ON TABLE user COLUMNS name FULLTEXT ANALYZER example_ascii BM25 HIGHLIGHTS;
