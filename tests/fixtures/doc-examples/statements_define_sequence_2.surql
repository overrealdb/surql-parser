DEFINE SEQUENCE seq;
CREATE my:counter SET val = 0;
sequence::nextval("seq");        -- 0
my:counter.val;                  -- 0

BEGIN TRANSACTION;
sequence::nextval("seq");        -- 0
UPDATE my:counter SET val += 1;  -- my:counter.val = 1
CANCEL TRANSACTION;              -- my:counter.val now rolled back to 0

sequence::nextval("seq");        -- 2
UPDATE my:counter SET val += 1;  -- 1
