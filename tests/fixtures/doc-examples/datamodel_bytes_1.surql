/**[test]

[[test.results]]
value = "'cellar door'"

[[test.results]]
value = "[72, 111, 98, 98, 105, 116, 115]""

*/

-- array<int> to bytes to string
<string><bytes>[ 99, 101, 108, 108, 97, 114, 32, 100, 111, 111, 114 ];
-- string to bytes to array<int>
<array><bytes>"Hobbits";
