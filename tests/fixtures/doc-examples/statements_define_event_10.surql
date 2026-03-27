DEFINE EVENT start ON start THEN {
    CREATE cat;
};

DEFINE EVENT cat ON person ASYNC MAXDEPTH 4 THEN {
  CREATE |cat:9|;  
};

DEFINE EVENT person ON cat ASYNC MAXDEPTH 4 THEN {
  CREATE |person:9|;
};

CREATE start;

count(SELECT VALUE id FROM person, cat);
