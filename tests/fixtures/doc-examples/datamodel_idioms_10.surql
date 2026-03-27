CREATE test:1 SET obj.a = 'a';

// Error
"Couldn't coerce value for field `obj.*` of `test:1`: Expected `number` but found `'a'`"
