-- Checks the index for ["Mandy", "wizard"], no existing
-- record found so no problem
UPSERT user SET name = "Mandy", class = "wizard";

-- Fails because statement tries to upsert a new user:mandy
-- on top of the previous randomly generated one
UPSERT user:mandy SET 
	name = "Mandy",
	class = "wizard",
	metadata = { likes: ["strawberries" ]};
