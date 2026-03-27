SELECT * FROM person; -- Error
DELETE person;        -- Error
CREATE person:one;    -- Succeeds
DELETE person;        -- Empty array
REMOVE TABLE person;  -- NONE
