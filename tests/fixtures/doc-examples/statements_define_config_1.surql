/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ headers: { "access-control-allow-origin": '*' }, raw: false, status: 200 }"

[[test.results]]
value = "{ body: { some: 'data' }, headers: { "access-control-allow-origin": '*' }, raw: false, status: 200 }"

*/

DEFINE CONFIG API
    MIDDLEWARE 
        api::res::headers({
            'Access-Control-Allow-Origin': '*'
        });

DEFINE API "/test" FOR get THEN {};
DEFINE API "/test2" FOR get THEN {
    {
        body: {
            some: "data"
        }
    }
};

api::invoke("/test");
api::invoke("/test2");
