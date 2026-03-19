/**[test]
[[test.results]]
value = "['0: first used in the year 876', '1: the number of moons in the sky', '2: also called a pair']"

*/

[
  ": first used in the year 876",
  ": the number of moons in the sky",
  ": also called a pair"
]
  .map(|$item, $index| <string>$index + $item);
