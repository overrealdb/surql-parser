-- Create a new record
CREATE user:john SET name = 'John' VERSION d'2025-08-19T08:00:00Z';
[[{ id: user:john, name: 'John' }]]

-- Select the record as it is now
SELECT * FROM user:john;
[[{ id: user:john, name: 'John' }]]

-- Select the record as it was at a specific point in time
SELECT * FROM user:john VERSION d'2025-08-19T08:00:00Z';
[[{ id: user:john, name: 'John' }]]

-- Select the record as it was at a specific point in time that doesn't exist
SELECT * FROM user:john VERSION d'2025-08-19T07:00:00Z';
[[]]

-- Update the record to the user john
update user:john Set hight ="55"
[[{ hight: '55', id: user:john, name: 'John' }]]

-- Confirm that the record is updated
SELECT * FROM user:john;
[[{ hight: '55', id: user:john, name: 'John' }]]

-- Select the record for the timestamp before the update
SELECT * FROM user:john VERSION d'2025-08-19T08:00:00Z';
[[{ id: user:john, name: 'John' }]]
