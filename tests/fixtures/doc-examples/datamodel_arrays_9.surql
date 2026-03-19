/**[test]

[[test.results]]
value = "['At index 0 we got a 1!', 'At index 1 we got a 2!', 'At index 2 we got a 3!']"

*/

[1,2,3].map(|$v, $i| "At index " + <string>$i + " we got a " + <string>$v + "!");
