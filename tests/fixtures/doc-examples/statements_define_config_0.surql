/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE CONFIG API
    MIDDLEWARE 
        api::timeout(10s),
        api::res::headers({
            'Access-Control-Allow-Origin': '*'
        });
