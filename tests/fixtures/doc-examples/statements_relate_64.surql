SELECT @.{1..5}.{ 
    id, 
    next_roads: ->to.*,
    next_cities: ->to->city
} FROM city;
