-- Don't return any result
UPDATE person SET skills += 'reading' RETURN NONE;

-- Return the changeset diff
UPDATE person SET skills += 'reading' RETURN DIFF;

-- Return the record before changes were applied
UPDATE person SET skills += 'reading' RETURN BEFORE;

-- Return the record after changes were applied (the default)
UPDATE person SET skills += 'reading' RETURN AFTER;

-- Return the value of the 'skills' field without the field name
UPDATE person SET skills += 'reading' RETURN VALUE skills;

-- Return a specific field only from the updated records
UPDATE person:tobie SET skills = ['skiing', 'music'] RETURN name, interests;
