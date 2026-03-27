FOR $_ IN 0..10000 {
  CREATE player:['mage', rand::id()] SET class = 'mage';
  CREATE player:['barbarian', rand::id()] SET class = 'barbarian';
  CREATE player:['rogue', rand::id()]     SET class = 'rogue';
  CREATE player:['bard', rand::id()]      SET class = 'bard';
  CREATE player:['sage', rand::id()]      SET class = 'sage';
  CREATE player:['psionic', rand::id()]   SET class = 'psionic';
  CREATE player:['thief', rand::id()]     SET class = 'thief';
  CREATE player:['paladin', rand::id()]   SET class = 'paladin';
  CREATE player:['ranger', rand::id()]    SET class = 'ranger';
  CREATE player:['cleric', rand::id()]    SET class = 'cleric';
};

LET $_ = SELECT * FROM player:['mage', NONE]..['mage', ..];
LET $_ = SELECT * FROM player WHERE class = 'mage';
