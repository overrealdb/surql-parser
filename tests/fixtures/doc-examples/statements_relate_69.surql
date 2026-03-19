SELECT 
	id, 
	->(SELECT out.id AS counterpart, !!dated AS dated FROM knows) AS acquaintances
FROM person;
