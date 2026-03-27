SELECT * FROM person;

-- Field `address` now shows up as "string::uppercase"
-- name.first structure now flattened into a simple field
SELECT
	name.first AS user_name,
	string::uppercase(address)
FROM person;

-- "Morgan Hitchcock" added to `name` field structure,
-- `angry_address` for field name instead of automatically
-- generated "string::uppercase(address) + '!!!'"
SELECT
	name.first,
	"Morgan Hitchcock" AS name.last,
	string::uppercase(address) + "!!!" AS angry_address
FROM person;
