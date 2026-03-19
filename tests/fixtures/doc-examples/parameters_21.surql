/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ language_en: 'DEFINE TABLE language_en TYPE NORMAL SCHEMAFULL PERMISSIONS NONE', language_ie: 'DEFINE TABLE language_ie TYPE NORMAL SCHEMAFULL PERMISSIONS NONE', language_ja: 'DEFINE TABLE language_ja TYPE NORMAL SCHEMAFULL PERMISSIONS NONE', language_uk: 'DEFINE TABLE language_uk TYPE NORMAL SCHEMAFULL PERMISSIONS NONE', user: "DEFINE TABLE user TYPE NORMAL SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE $access = 'users' AND email = $token.email" }"

*/

FOR $language IN ["en", "ja", "uk", "ie"] {
    DEFINE TABLE "language_" + $language SCHEMAFULL;
};

(INFO FOR DB).tables;
