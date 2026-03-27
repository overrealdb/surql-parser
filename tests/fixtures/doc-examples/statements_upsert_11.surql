DEFINE INDEX unique_key ON TABLE user FIELDS name, class UNIQUE;
DEFINE FIELD official_name ON TABLE user VALUE name + " the " + class;

-- Add 50000 users to fill up the database
FOR $i IN <array>0..50000 { CREATE user SET name = <string>$i, class = <string>$i; };

-- Create Billy, one of 50,001 records
CREATE user SET name = "Billy", class = "wizard";

-- Updating Billy requires a table scan
UPDATE user SET
	interests += "music"
WHERE
	name = "Billy" AND
	class = "wizard";

-- But UPSERT uses 'name' and 'class' to check the index anyway,
-- and thus can use it to access the record without a scan
UPSERT user SET
	name = "Billy",
	class = "wizard",
	interests += "travel";
