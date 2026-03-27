SELECT 
	-- Show all `country` records located at `out`
	->has->country AS countries,
	-- Show all `province` or `state` records located at `out`	
	->has->country->has->(province, state) AS state_provinces,
	-- Or use (?) to show any type of record located at `out`
	->has->(?)->has->(?)->has->(?) AS cities
FROM planet:earth;
