DEFINE API "/always_ok"
    FOR get
        MIDDLEWARE
            api::res::status(200)
        THEN {
            {
                status: 404,
                body: {
                    some: "data"
                }
            };
        };

api::invoke("/always_ok");
