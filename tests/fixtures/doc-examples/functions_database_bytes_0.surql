/**[test]

[[test.results]]
value = "[19, 67, 25]"

*/

RETURN [
    bytes::len(<bytes>"Simple ASCII string"),
    bytes::len(<bytes>"οὐ γὰρ δυνατόν ἐστιν ἔτι καθεύδειν"),
    bytes::len(<bytes>"청춘예찬 靑春禮讚")
];
