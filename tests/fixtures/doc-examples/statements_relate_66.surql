CREATE |person:1..4|;
FOR $person IN (SELECT * FROM person) {
  LET $others = (SELECT * FROM person WHERE id != $person.id);
    FOR $other IN $others {
        RELATE $person->likes->$other;
    }
};
RETURN [
	person:1.{2}->likes->person,
	person:1.{3}->likes->person,
	person:1.{4}->likes->person
];
