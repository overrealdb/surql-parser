-- This will set the value of the `primary` field to `123.456`
CREATE product:test;

-- This will set the value of the `primary` field to `463.456`
UPSERT product:test SET primary = 463.456;

-- This will set the value of the `primary` field to `123.456`
UPSERT product:test SET primary = NONE;
