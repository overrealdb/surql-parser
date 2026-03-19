/**[test]

[[test.results]]
value = "[]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ data_length: 3, id: person:one, is_short: true, random_data: 'HI!' }]"

[[test.results]]
value = "[{ data_length: 3, id: person:one, is_short: true, random_data: 'HI!' }]"

[[test.results]]
value = "[{ data_length: 3, id: person:one, is_short: true, random_data: 'HI!' }]"

[[test.results]]
value = "[{ data_length: 3, id: person:one, is_short: true, random_data: 'HI!' }]"

*/

DEFINE FIELD data_length ON person VALUE random_data.len();
DEFINE FIELD is_short ON person VALUE random_data.len() < 10;

-- Fill up the database a bit with 10,000 records
CREATE |person:10000| SET random_data = rand::string(1000) RETURN NONE;
-- Add one outlier with short random_data
CREATE person:one SET random_data = "HI!" RETURN NONE;

-- Function call + compare operation: slowest
SELECT * FROM person WHERE random_data.len() < 10;
-- Compare operation: much faster
SELECT * FROM person WHERE data_length < 10;
-- Boolean check: even faster
SELECT * FROM person WHERE is_short;
-- Direct record access: almost instantaneous
SELECT * FROM person:one;
