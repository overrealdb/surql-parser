-- Start with you
person:you
-- Get the shortest path
	.{..+shortest=person:star+inclusive}
-- by following ->knows->person
	(->knows->person)
-- then grab the names
	.name
-- and capitalize each one
	.map(|$n| $n.uppercase());
