RELATE character:one->speaks_to:ulid()->character:two SET content = "Greetings, adventurer!";
RELATE character:one->speaks_to:ulid()->character:two SET content = "Can you please help me? My sheep have run amok.";

SELECT
	// Grab the latter part of the record ID, turn it into a datetime
    time::from_ulid(id.id()) AS at,
    content
FROM
    // ULID from 2025-04-25, well before today's date
    character:one->speaks_to:01JSNG0KZSY3HJ5QSZ7JSMQMGR..;
