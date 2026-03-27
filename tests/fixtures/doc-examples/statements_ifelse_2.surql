/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "'Positive uninteresting number'"

*/

LET $num = 100;

RETURN IF $num < 0 {
    "Negative"
} ELSE IF $num = 0 {
    "Zero"
} ELSE IF $num = 13 {
    "Thirteen"
} ELSE {
    "Positive uninteresting number"
};
