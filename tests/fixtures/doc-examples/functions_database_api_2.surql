/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE API "/test"
    FOR get 
        MIDDLEWARE
            api::timeout(1s)
        THEN {
            {
                status: 404,
                body: $request.body,
                headers: { the_time_is_now: <string>time::now() }
            };
        };
