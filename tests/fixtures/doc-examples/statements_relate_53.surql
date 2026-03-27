CREATE person:one, person:two;
-- These three relations would end up deleted
RELATE person:one->friends_with->person:two;
RELATE person:one->dating->person:two;
RELATE person:one->hates->person:two;
-- Finally this would be the graph edge connecting the two
RELATE person:one->married->person:two;
