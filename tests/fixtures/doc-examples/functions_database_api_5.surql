/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE API "/exceeds_timeout"
    FOR get 
        MIDDLEWARE
            api::timeout(1ns)
        THEN {
            sleep 1ns;
            {}
        };
