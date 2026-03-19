SELECT id, array::complement(<->sister_of<->city, [id]) AS sister_cities FROM city;
