UPDATE person 
	SET important = true 
	WHERE ->knows->person->(knows WHERE influencer = true) 
	TIMEOUT 5s;
