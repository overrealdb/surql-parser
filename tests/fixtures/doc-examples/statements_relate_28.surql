-- This query
SELECT ->purchased->product<-purchased<-person->purchased->product FROM person:tobie

-- Then becomes
person:tobie->purchased->product<-purchased<-person->purchased->product SELECT
