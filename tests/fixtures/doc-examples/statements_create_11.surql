/**[test]

[[test.results]]
value = "[[{ op: 'replace', path: '', value: { age: 46, id: person:2wm77z4dn1wjh7az4r5f, username: 'john-smith' } }]]"

*/

CREATE person SET age = 46, username = "john-smith" RETURN DIFF;
