/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE API "/test"
  FOR get 
    MIDDLEWARE
      api::res::header("country-origin", "CA")
    THEN {
      {
        status: 200,
        headers: {
          "requested-at": <string>time::now()
        },
        body: SELECT * FROM person
      };
    };
