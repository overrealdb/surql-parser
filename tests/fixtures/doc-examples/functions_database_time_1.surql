/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[d'2024-08-30T03:00:00Z', d'2024-09-05T00:00:00Z']"

*/
LET $now = d'2024-08-30T02:22:50.231631Z';

RETURN [
  time::ceil($now, 1h),
  time::ceil($now, 1w)
];
