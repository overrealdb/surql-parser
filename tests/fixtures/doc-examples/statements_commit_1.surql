/**[test]

[[test.results]]
value = "'Money sent! Status:
{ balance: 135905.16f, id: account:one }
{ balance: 90731.31f, id: account:two, wants_to_send_money: true }'"

[[test.results]]
value = "{ balance: 135905.16f, id: account:one }"

[[test.results]]
value = "{ balance: 90731.31f, id: account:two, wants_to_send_money: true }"

*/

BEGIN;

CREATE account:one SET balance = 135605.16;
CREATE account:two SET balance = 91031.31, wants_to_send_money = true;

IF !account:two.wants_to_send_money {
    THROW "Customer doesn't want to send any money!";
};

LET $first = UPDATE ONLY account:one SET balance += 300.00;
LET $second = UPDATE ONLY account:two SET balance -= 300.00;

RETURN "Money sent! Status:\n" + <string>$first + '\n' + <string>$second;

COMMIT;
