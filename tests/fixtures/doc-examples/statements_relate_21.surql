LET $road = SELECT * FROM ONLY road WHERE name = "Dalhurst Way" LIMIT 1;
RELATE [
    house:["Dalhurst Way", 218],
    house:["Dalhurst Way", 222],
    house:["Dalhurst Way", 226],
]->has_road->$road;
