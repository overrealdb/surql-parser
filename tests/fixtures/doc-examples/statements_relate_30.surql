-- Select products purchased by people in the last 3 weeks who have purchased the same products that tobie purchased
SELECT 
	->purchased->product
	<-purchased<-person->(purchased WHERE created_at > time::now() - 3w)
	->purchased->product
FROM person:tobie;
