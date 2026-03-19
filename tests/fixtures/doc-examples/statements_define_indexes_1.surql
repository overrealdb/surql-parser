/**[test]

[[test.results]]
value = "NONE"

*/

-- Makes sure that the email address in the user table is always unique
DEFINE INDEX userEmailIndex ON TABLE user COLUMNS email UNIQUE;
