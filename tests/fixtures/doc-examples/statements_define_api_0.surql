/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE API "/test"
    FOR get, post 
        MIDDLEWARE
            api::timeout(1s)
        THEN {
            {
                status: 200,
                body: {
                    request: $request.body,
                    response: "The server works"
                },
                headers: {
                    'last-modified': <string>time::now(),
                    'expires': <string>(time::now() + 4d)
                }
            };
        };
