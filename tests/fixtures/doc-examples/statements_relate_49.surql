CREATE country:usa SET name = "USA";
CREATE state:pennsylvania SET population = 12970000;
CREATE state:michigan SET population = 10030000;
CREATE city:philadelphia, city:pittsburgh, city:detroit, city:grand_rapids;

RELATE country:usa->contains->[state:pennsylvania, state:michigan];
RELATE state:pennsylvania->contains->[city:philadelphia, city:pittsburgh];
RELATE state:michigan->contains->[city:detroit, city:grand_rapids];
