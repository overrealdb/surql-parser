/**[test]

[[test.results]]
value = "[{ "[math::min(temps), math::max(temps)]": [-5, 9], "math::mean": 4f }]"

*/

SELECT
    math::mean(temps),
    [ math::min(temps), math::max(temps) ]
FROM { temps: [-5, 8, 9] };
