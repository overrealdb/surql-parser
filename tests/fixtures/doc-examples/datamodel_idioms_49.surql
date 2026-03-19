CREATE |person:1..21| SET name = id.id() RETURN NONE;
FOR $person IN SELECT * FROM person {
    LET $friends = (SELECT * FROM person WHERE id != $person.id ORDER BY rand() LIMIT 2);
    RELATE $person->friends_with->$friends;
};

LET $third_degree = person:1.{..3}.{ id, connections: ->friends_with->person.@ };
// Object containing array of arrays of arrays of 'person'
RETURN $third_degree;
// All connections: an array of arrays of arrays of 'person'
RETURN $third_degree.connections;
// Secondary connections: an array of arrays of 'person'
RETURN $third_degree.{2}.connections;
// Tertiary connections: an array of 'person'
RETURN $third_degree.{3}.connections;
// Tertiary connections with aliased fields and original 'person' info
RETURN $third_degree.{
		original_person: id, 
		third_degree_friends: connections.{2}.connections
};
