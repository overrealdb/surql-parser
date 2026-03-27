DEFINE EVENT one ON account ASYNC RETRY 0 THEN {
    IF rand::bool() {
        THROW "Failed!"
    } ELSE {
        CREATE it:worked SET very = "well";
    }
};

DEFINE EVENT two ON account ASYNC RETRY 0 THEN {
    IF rand::bool() {
        THROW "Failed!"
    } ELSE {
        CREATE it:worked SET very = "well";
    }
};

CREATE account;
