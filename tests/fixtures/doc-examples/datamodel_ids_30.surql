-- Create records from person:1 to person:10
CREATE |person:1..11|;
LET $latest = SELECT VALUE id FROM ONLY person ORDER BY id DESC LIMIT 1;
CREATE type::record("person", $latest.id() + 1);
