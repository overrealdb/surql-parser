/**[test]

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

*/

BEGIN TRANSACTION;

-- Setup accounts
CREATE account:one SET balance = 135605.16;
CREATE account:two SET balance = 91031.31;

-- Move money
UPDATE account:one SET balance += 300.00;
UPDATE account:two SET balance -= 300.00;

-- Rollback all changes
CANCEL TRANSACTION;
