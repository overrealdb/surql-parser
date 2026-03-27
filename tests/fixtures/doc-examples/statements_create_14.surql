/**[test]

[[test.results]]
value = "[{ age: 46, age_next_year: 47, interests: ['skiing', 'music'] }]"

[[test.results]]
value = "[20, 20, 20, 20, 20]"

*/

CREATE person
    SET age = 46,
    username = "john-smith",
    interests = ['skiing', 'music']
RETURN
    age,
    interests,
    age + 1 AS age_next_year;

CREATE |person:5|
    SET age = 20
RETURN VALUE age;
