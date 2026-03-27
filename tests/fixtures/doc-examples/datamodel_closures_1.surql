/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "'Hello, Alice!'"

*/

-- Define a function that greets a person
LET $greet = |$name: string| -> string { "Hello, " + $name + "!" };
RETURN $greet("Alice");   -- Returns "Hello, Alice!"
