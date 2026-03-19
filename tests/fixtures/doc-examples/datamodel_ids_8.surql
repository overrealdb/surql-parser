FOR $_ IN 0..10000 {
    CREATE player:['mage', rand::id()] SET class = 'mage';
};

LET $_ = SELECT * FROM player:['mage', NONE]..['mage', ..];
LET $_ = SELECT * FROM player WHERE class = 'mage';
