/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "4"

[[test.results]]
value = "NONE"

[[test.results]]
value = "'Hello, World!'"

*/

-- Define an anonymous function that doubles a number
LET $double = |$n: number| $n * 2;
RETURN $double(2);  -- Returns 4

-- Define a function that concatenates two strings
LET $concat = |$a: string, $b: string| $a + $b;
RETURN $concat("Hello, ", "World!");  -- Returns "Hello, World!"
