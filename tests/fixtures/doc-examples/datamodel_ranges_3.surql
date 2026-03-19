/**[test]

[[test.results]]
value = "NONE"

*/

FOR $year IN 0..=2024 {
    CREATE historical_events SET
        for_year = $year,
        events = "To be added";
}
