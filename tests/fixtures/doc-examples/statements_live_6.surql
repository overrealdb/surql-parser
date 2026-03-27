LET $table = 'measurement';
LET $location = 'Tallinn';
LIVE SELECT * FROM type::table($table) WHERE location == $location;
