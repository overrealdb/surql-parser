/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE API "/test"
    FOR get 
        MIDDLEWARE
            api::res::headers({
                "country-origin": "CA",
                "language": "FR"
            })
        THEN {
            {
                status: 200,
                headers: {
                    "requested-at": <string>time::now()
                },
                body: SELECT * FROM person
            };
        };
