DEFINE API "/status/invalid-low"
    FOR get
        MIDDLEWARE
            api::res::status(99)
        THEN {
            RETURN {
                status: 200,
                body: {}
            };
        };

api::invoke("/status/invalid-low");
