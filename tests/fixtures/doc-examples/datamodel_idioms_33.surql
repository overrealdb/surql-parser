SELECT 
	next AS countries,
	next.next AS states_provinces,
	next.next.next AS cities
FROM planet:earth;
