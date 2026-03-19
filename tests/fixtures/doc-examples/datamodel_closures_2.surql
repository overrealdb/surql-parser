/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "'HELLO'"

[[test.results]]
error = "'Incorrect arguments for function ANONYMOUS(). Expected a value of type 'string' for argument $text'"

[[test.results]]
value = "NONE"

[[test.results]]
value = "16"

[[test.results]]
error = "'Incorrect arguments for function ANONYMOUS(). Expected a value of type 'number' for argument $num'"

*/

-- Define a function with a return type
LET $to_upper = |$text: string| -> string { string::uppercase($text) };
RETURN $to_upper("hello");  -- Returns "HELLO"
RETURN $to_upper(123);      -- Error: type mismatch

-- Define a function that accepts only numbers
LET $square = |$num: number| $num * $num;
RETURN $square(4);    -- Returns 16
RETURN $square("4");  -- Error: type mismatch
