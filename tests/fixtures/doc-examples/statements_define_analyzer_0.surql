/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "['BDlaerruS', 'ekil', 'I']"

*/

DEFINE FUNCTION fn::backwardsify($input: string) -> string {
    $input.split('').fold('', |$a, $b| $b + $a);
};

DEFINE ANALYZER backwards FUNCTION fn::backwardsify TOKENIZERS blank;

search::analyze("backwards", "I like SurrealDB");
