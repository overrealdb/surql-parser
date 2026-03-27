-- Starting with Tobie
person:tobie
-- move on to his purchased products
->purchased->product
-- that were also purchased by persons...
<-purchased<-person
-- what are all of those persons' purchased products?
->purchased->product
