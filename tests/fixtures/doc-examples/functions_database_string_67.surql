/**[test]

[[test.results]]
value = ""I'LL SEND YOU A CHEQUE FOR THE CATALOGUE!!!!""

*/

-- Traditional syntax
string::concat(
  string::uppercase(
    string::replace(
      string::replace("I'll send you a check for the catalog", "ck", "que")
    , "og", "ogue")
  )
, "!!!!");

-- Method chaining syntax
"I'll send you a check for the catalog"
  .replace("ck", "que")
  .replace("og", "ogue")
  .uppercase()
  .concat("!!!!");
