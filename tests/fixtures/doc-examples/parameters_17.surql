DEFINE API OVERWRITE "/test"
    FOR get, post 
        MIDDLEWARE
            api::timeout(1s)
        THEN {
            RETURN {
                status: 404,
                body: $request.body,
                headers: {
                    'bla': '123'
                }
            };
        };
