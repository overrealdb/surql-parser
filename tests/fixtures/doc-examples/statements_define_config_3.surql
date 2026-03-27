DEFINE CONFIG API
    MIDDLEWARE 
        api::timeout(10s),
        api::res::headers({
            'Access-Control-Allow-Origin': '*'
        });

DEFINE API OVERWRITE "/test"
    FOR get 
        MIDDLEWARE
            api::timeout(1µs)
        THEN {
            RETURN {
                status: 200,
                body: { 
                    data: (SELECT * FROM person),
                    however: "This will probably never return because the timeout is 1 microsecond"
                }
            };
        };

api::invoke("/test");
