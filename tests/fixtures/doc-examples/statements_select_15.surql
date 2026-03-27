/**[test]

[[test.results]]
value = "[{ age: 20, country: 'Japan', gender: 'M', id: person:btiyllegvd4qiy1vspj8 }, { age: 25, country: 'Japan', gender: 'M', id: person:mf73xwobyhqyxc4suxdl }, { age: 23, country: 'US', gender: 'F', id: person:45u27tit21osibawb3zm }, { age: 30, country: 'US', gender: 'F', id: person:t950ztxmelm5nlqzuplz }, { age: 25, country: 'Korea', gender: 'F', id: person:t8toz5e3muclr3hxe74z }, { age: 45, country: 'UK', gender: 'F', id: person:s6mme1k3txzppbg5zx92 }]"
skip-record-id-key = true

[[test.results]]
value = "[{ average_age: 25, country: 'Korea', gender: 'F', total: 1 }, { average_age: 45, country: 'UK', gender: 'F', total: 1 }, { average_age: 26.5f, country: 'US', gender: 'F', total: 2 }, { average_age: 22.5f, country: 'Japan', gender: 'M', total: 2 }]"

[[test.results]]
value = "[{ number_of_records: 6 }]"

*/

INSERT INTO person [
    { gender: "M", age: 20, country: "Japan" },
    { gender: "M", age: 25, country: "Japan" },
    { gender: "F", age: 23, country: "US" },
    { gender: "F", age: 30, country: "US" },
    { gender: "F", age: 25, country: "Korea" },
    { gender: "F", age: 45, country: "UK" },
];

SELECT
	count() AS total,
	math::mean(age) AS average_age,
	gender,
	country
FROM person
GROUP BY gender, country;

-- Get the total number of records in a table
SELECT count() AS number_of_records FROM person GROUP ALL;
