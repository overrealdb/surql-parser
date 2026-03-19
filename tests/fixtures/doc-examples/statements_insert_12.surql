/**[test]

[[test.results]]
value = "[{ age: 46, age_next_year: 47, interests: ['skiing', 'music'] }]"

[[test.results]]
value = "[27, -55]"

*/

INSERT INTO person {
    age: 46,
    username : "john-smith",
    interests : ['skiing', 'music'] }
RETURN
    age,
    interests,
    age + 1 AS age_next_year;

INSERT INTO planet [
	{
		name: 'Venus',
        surface_temp: 462,
        temp_55_km_up: 27
	},
	{
		name: 'Earth',
        surface_temp: 15,
        temp_55_km_up: -55
	}
] RETURN VALUE temp_55_km_up;
