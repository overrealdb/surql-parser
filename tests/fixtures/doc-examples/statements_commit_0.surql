/**[test]

[[test.results]]
value = "[{ balance: 135605.16f, id: account:one }]"

[[test.results]]
value = "[{ balance: 91031.31f, id: account:two }]"

[[test.results]]
value = "[{ balance: 135905.16f, id: account:one }]"

[[test.results]]
value = "[{ balance: 90731.31f, id: account:two }]"

*/

BEGIN TRANSACTION;

-- Setup accounts
CREATE account:one SET balance = 135605.16;
CREATE account:two SET balance = 91031.31;

-- Move money
UPDATE account:one SET balance += 300.00;
UPDATE account:two SET balance -= 300.00;

-- Finalise all changes
COMMIT TRANSACTION;
