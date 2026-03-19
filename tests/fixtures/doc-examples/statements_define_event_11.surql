-- Create five people
CREATE |person:1..=5|;
-- Make each person friends with each of the four others
UPDATE person SET friends = (SELECT VALUE id FROM person).complement([$this.id]);
-- Count after five levels of depth is already 1024!
count(person:1.{..5}.friends);
