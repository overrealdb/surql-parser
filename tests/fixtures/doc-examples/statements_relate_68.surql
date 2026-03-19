CREATE person:one, person:two, person:three;
RELATE person:one->knows->person:two SET
	friends = true,
    dated = true,
    married_to = true;

RELATE person:one->knows->person:three SET
    dated = true;

RELATE person:two->knows->person:three SET
	friends = true;
