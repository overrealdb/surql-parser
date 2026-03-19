-- Create a record for user:john at 8:00AM
CREATE user:john SET name = 'John' VERSION d'2024-08-19T08:00:00Z';
[[{ id: user:john, name: 'John' }]]

-- Return the record for user:john at 8:00AM
SELECT * FROM user:john VERSION d'2024-08-19T08:00:00Z';
[[{ id: user:john, name: 'John' }]]

-- Create a record for user:john at 8:01AM
CREATE user:john SET name = 'John-1' VERSION d'2024-08-19T08:01:00Z';
[[{ id: user:john, name: 'John-1' }]]

-- Return the record for user:john at 8:01AM
SELECT * FROM user:john VERSION d'2024-08-19T08:01:00Z';
[[{ id: user:john, name: 'John-1' }]]

-- Return an empty array because the record at the datetime does not exist
SELECT * FROM user:john VERSION d'2024-08-19T07:00:00Z';
[[]]
