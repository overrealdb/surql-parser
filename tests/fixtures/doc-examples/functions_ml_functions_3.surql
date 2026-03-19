SELECT * FROM (
		SELECT 
			*, 
			ml::house-price-prediction<0.0.1>({ squarefoot: squarefoot_col, num_floors: num_floors_col }) AS price_prediction 
		FROM house_listing
	) 
	WHERE price_prediction > 177206.21875;
