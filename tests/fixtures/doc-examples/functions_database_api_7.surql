DEFINE API "/serialize_json"
    FOR get
        MIDDLEWARE
            api::res::body("json")
        THEN {
            {
                status: 200,
                body: {
                    message: "Hello"
                }
            };
        };

api::invoke("/serialize_json").{
    body: <string>body, -- Cast response bytes into string
    headers,
    status
};
