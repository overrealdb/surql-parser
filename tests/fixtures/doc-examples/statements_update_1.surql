-- Update all records in a table
-- The `enjoys` field will also be an array.
-- The += operator alone is enough to infer the type
UPDATE person SET 
	dollars = 50,
	skills += 'breathing',
	enjoys += 'reading',
	full_name = name + ' Mc' + name + 'erson';
