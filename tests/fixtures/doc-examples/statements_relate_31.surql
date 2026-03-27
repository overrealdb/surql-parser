SELECT 
	->purchased->(product, subscription)
	<-purchased<-person
	->purchased->(product, subscription)
FROM person:tobie;
