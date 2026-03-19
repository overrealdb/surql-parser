/**[test]

[[test.results]]
value = "[true, true, true, true, true, true, true, true, true, true, true, true]"

*/

RETURN [
    null > none,
    true > null,
    1 > true,
    'a' > 999999999,
    1m > 'a',
    time::now() > 1m,
    rand::uuid() > time::now(),
    [ 9, 10 ] > rand::uuid(),
    { 9, 10 } > [ 9, 10 ],
    {} > { 9, 10 },
    (9.9, 9.9) > {},
    <bytes>"Aeon" > (9.9, 9.9),
    type::table("person") > <bytes>"Aeon",
    person:one > type::table("person"),
    f"file://myfile.txt" > person:one,
    <regex>"a|b" > f"file://myfile.txt",
    0..10 > <regex>"a|b",
    || > 0.. 10
];
