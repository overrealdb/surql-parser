SELECT 
	->knows->person AS knows,
	->knows[WHERE has_dated]->person AS has_dated,
	->knows[WHERE same_high_school AND has_dated]->person AS dated_and_same_school
 FROM person:one;
