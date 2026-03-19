CREATE |person:1..4|;

RELATE person:1->likes->person:2 SET like_strength = 20, know_in_person = true;
RELATE person:1->likes->person:3 SET like_strength = 5,  know_in_person = false;
RELATE person:2->likes->person:1 SET like_strength = 10, know_in_person = true;
RELATE person:2->likes->person:3 SET like_strength = 12, know_in_person = false;
RELATE person:3->likes->person:1 SET like_strength = 2,  know_in_person = false;
RELATE person:3->likes->person:2 SET like_strength = 9,  know_in_person = false;

SELECT ->likes AS likes FROM person;
SELECT ->(SELECT like_strength FROM likes) AS likes FROM person;
SELECT ->(SELECT like_strength FROM likes WHERE like_strength > 10) AS likes FROM person;
SELECT ->(likes WHERE like_strength > 10) AS likes FROM person;
SELECT ->(SELECT like_strength, know_in_person FROM likes ORDER BY like_strength DESC) AS likes FROM person;
SELECT ->(SELECT count() as count, know_in_person FROM likes GROUP BY know_in_person) AS likes FROM person;
SELECT ->(likes LIMIT 1) AS likes FROM person;
SELECT ->(likes START 1) AS likes FROM person;
