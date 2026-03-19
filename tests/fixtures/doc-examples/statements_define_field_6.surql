/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Set a field to have the array data type
DEFINE FIELD roles ON TABLE user TYPE array<string>;

-- Set a field to have the array data type, equivalent to `array<any>`
DEFINE FIELD posts ON TABLE user TYPE array;

-- Set a field to have the array object data type
DEFINE FIELD emails ON TABLE user TYPE array<object>;

-- Set a field that holds exactly 640 bytes
DEFINE FIELD bytes ON TABLE data TYPE array<int, 640> ASSERT $value.all(|$val| $val IN 0..=255);

-- Field for a block in a game showing the possible distinct directions a character can move next.
-- The array can contain no more than four directions
DEFINE FIELD next_paths ON TABLE block 
  TYPE array<"north" | "east" | "south" | "west"> 
  VALUE $value.distinct() 
  ASSERT $value.len() <= 4;
