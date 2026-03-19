/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

*/


-- All true
'g' IN 'a'..'z';
d"2024-01-01" IN d"2020-01-01"..=d"2025-01-01";
['London', d"2022-02-02", 5.7] IN ['London', d"2020-01-01"]..=['London', d"2024-12-31"];

-- All false
"ㅋㅋㅋ" IN "a".."z";
d"2028-01-01" IN d"2020-01-01"..=d"2025-01-01";
['Philadelphia', d"2022-02-02", 5.7] IN ['London', d"2020-01-01"]..=['London', d"2024-12-31"];
