BEGIN;
RETURN 1; -- Is executed
CREATE a; -- Is not executed
RETURN 2; -- Is not executed
COMMIT;
