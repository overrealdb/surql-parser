/**
[test]

[[test.results]]
value = "d'2025-11-28T11:41:20.262Z'"

[[test.results]]
value = "d'2025-11-28T07:41:20.262Z'"

[[test.results]]
value = "d'2025-11-28T15:41:20.262Z'"

[[test.results]]
value = "d'2025-11-28T11:41:20Z'"

[[test.results]]
value = "d'2025-11-28T07:41:20Z'"

**/

RETURN d"2025-11-28T11:41:20.262Z";       --- Sub-second precision included, timezone defaulted to UTC
RETURN d"2025-11-28T11:41:20.262+04:00";  --- Sub-second precision included, timezone specified as UTC + 4:00
RETURN d"2025-11-28T11:41:20.262-04:00";  --- Sub-second precision included, timezone specified as UTC - 4:00
RETURN d"2025-11-28T11:41:20Z";           --- Sub-second precision excluded, timezone defaulted to UTC
RETURN d"2025-11-28T11:41:20+04:00";      --- Sub-second precision excluded, timezone specified as UTC + 4:00
