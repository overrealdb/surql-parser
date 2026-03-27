/**[test]

[[test.results]]
value = "2000"

*/

"Two"
    .replace("Two", "2")
    .chain(|$num| <number>$num * 1000);
