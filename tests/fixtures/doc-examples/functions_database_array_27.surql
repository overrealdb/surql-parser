/**[test]

[[test.results]]
value = "[{ importance: 10, message: 'I need some help with this query...' }, { importance: 100, message: 'Stuck on an island with two hours of battery life left. Can you...' }]"

*/

 [
    { importance: 10, message: "I need some help with this query..." },
    { importance: 0, message: "TEST Is this thing on?" },
    { importance: 5, message: "I have an idea. What if we..."},
    { importance: 100, message: "Stuck on an island with two hours of battery life left. Can you..."}
].filter(|$v| $v.importance > 5);
