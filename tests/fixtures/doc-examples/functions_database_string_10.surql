LET $input = "grey";
LET $string = "gr(a|e)y";
LET $regex = <regex>"gr(a|e)y";

[type::of($string), type::of($regex)];
-- ['string', 'regex']

[
  string::matches($input, $string),
  string::matches($input, $regex),
];
-- [true, true]
