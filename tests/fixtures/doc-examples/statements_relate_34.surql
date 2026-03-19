CREATE person:anakin_skywalker, person:luke_skywalker, person:the_emperor;
RELATE person:anakin_skywalker->served->person:the_emperor;
RELATE person:anakin_skywalker->attacked->person:the_emperor SET won = true;
RELATE person:the_emperor->attacked->person:luke_skywalker SET won = false;
RELATE person:luke_skywalker->son_of->person:anakin_skywalker;
RELATE person:the_emperor->fooled->person:anakin_skywalker SET date = "19 BBY";

-- As a SELECT statement
SELECT VALUE <->(?)[WHERE person:the_emperor IN [in, out]] FROM ONLY person:anakin_skywalker;
SELECT VALUE <->(?)[WHERE person:luke_skywalker IN [in, out]] FROM ONLY person:anakin_skywalker;

-- Or returned directly from the record ID
person:anakin_skywalker<->(?)[WHERE person:the_emperor IN [in, out]];
person:anakin_skywalker<->(?)[WHERE person:luke_skywalker IN [in, out]];
