/**[test]

[[test.results]]
error = "'Incorrect arguments for function geo::area(). Argument 1 was the wrong type. Expected `geometry` but found `12345`'"

*/
RETURN geo::area(12345);

-- 'Incorrect arguments for function geo::area(). Argument 1 was the wrong type. Expected `geometry` but found `12345`'
