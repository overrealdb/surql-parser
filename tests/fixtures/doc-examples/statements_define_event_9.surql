DEFINE TABLE did_not_throw;

DEFINE EVENT may_throw ON person ASYNC THEN {
  IF rand::bool() {
      THROW "This message will never show";
  } ELSE {
    CREATE did_not_throw;  
  }
};

CREATE |person:50|;
count(SELECT * FROM did_not_throw);
