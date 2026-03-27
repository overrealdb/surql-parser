-- This will return an error
CREATE product:test SET primary = NULL;

-- result 
"Couldn't coerce value for field `primary` of `product:test`: Expected `number` but found `NULL`"
