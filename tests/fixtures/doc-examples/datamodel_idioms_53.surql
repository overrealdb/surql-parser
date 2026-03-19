LET $you = SELECT VALUE id FROM ONLY person WHERE name = 'you' LIMIT 1;
LET $star = SELECT VALUE id FROM ONLY person WHERE name = 'star' LIMIT 1;
$you.{..+shortest=$star}->knows->person;
