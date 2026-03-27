-- Store the subquery result in a variable and query that result.
LET $avg_price = (
	SELECT math::mean(price) AS avg_price FROM product GROUP ALL
).avg_price;

-- Find the name of the product where the price is higher than the avg price
SELECT name FROM product
WHERE [price] > $avg_price;

-- Use the parent instance's field in a subquery (predefined variable)
SELECT *, (SELECT * FROM events WHERE host == $parent.id) AS hosted_events FROM user;
