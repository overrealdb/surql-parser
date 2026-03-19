/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE FIELD filter ON TABLE search_settings TYPE
      "None"
    | { type: "Ascii" }
    | { type: "EdgeNgram", from: int, to: int }
    | { type: "Lowercase" }
    | { type: "Ngram", from: int, to: int }
    | { type: "Snowball", language: string }
    | { type: "Uppercase" };
