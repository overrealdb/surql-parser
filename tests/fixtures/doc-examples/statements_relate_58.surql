-- Note: 1..6 used to be inclusive until SurrealDB 3.0.0
-- Now creates 1 up to but not including 6
CREATE |city:1..=6| SET name = <string>id.id() + 'ville';
FOR $pair IN (<array>(1..=5)).windows(2) {
  	LET $city1 = type::record("city", $pair[0]);
    LET $city2 = type::record("city", $pair[1]);
    RELATE $city1->to->$city2 SET 
        type = rand::enum(["train", "road", "bike path"]),
        distance = <int>(rand::float() * 100).ceil()
};
