/**[test]

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'The query was not executed due to a cancelled transaction'"

[[test.results]]
error = "'An error occurred: Not enough funds'"

*/

BEGIN TRANSACTION;

-- Setup accounts
CREATE account:one SET balance = 135605.16;
CREATE account:two SET balance = 200.31;

-- Move money
UPDATE account:one SET balance += 300.00;
UPDATE account:two SET balance -= 300.00;

IF account:two.balance < 0 {
    THROW "Not enough funds";
};

COMMIT TRANSACTION;
