/**[test]

[[test.results]]
value = "[{ avg_temps: [-5, 9], mean_temps: 4f }]"

*/

SELECT
    math::mean(temps) AS mean_temps,
    [ math::min(temps), math::max(temps) ] AS avg_temps
FROM { temps: [-5, 8, 9] };
