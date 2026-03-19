/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ body: { some: 'data' }, context: {  }, headers: {  }, status: 200 }"

[[test.results]]
value = "{ body: { some: 'data' }, context: {  }, headers: {  }, status: 200 }"

[[test.results]]
value = "{ body: { some: 'data' }, context: {  }, headers: {  }, status: 200 }"

*/

DEFINE API OVERWRITE "/test/*anything_goes" FOR get THEN {
    RETURN {
        body: {
            some: "data"
        }
    }
};

api::invoke("/test/this_matches");
api::invoke("/test/same_here");
api::invoke("/test/works/with/multiple/paths/now");
