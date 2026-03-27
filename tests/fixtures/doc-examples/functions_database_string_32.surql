/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN string::is_ascii("ABC123"); -- true
'𓀀'.is_ascii(); -- false
