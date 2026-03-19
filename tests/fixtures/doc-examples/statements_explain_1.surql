CREATE person:one, person:two;
RELATE person:one->friend->person:two;

EXPLAIN SELECT ->friend->person AS friends FROM ONLY person:one;
EXPLAIN person:one.{ friends: ->friend->person };
