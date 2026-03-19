/**[test]

[[test.results]]
error = ""Thrown error: 'access' is a protected variable and cannot be set""

[[test.results]]
error = ""Thrown error: 'auth' is a protected variable and cannot be set""

[[test.results]]
error = ""Thrown error: 'token' is a protected variable and cannot be set""

[[test.results]]
error = ""Thrown error: 'session' is a protected variable and cannot be set""

*/

LET $access = true;
LET $auth = 10;
LET $token = "Mytoken";
LET $session = rand::int(0, 100);
