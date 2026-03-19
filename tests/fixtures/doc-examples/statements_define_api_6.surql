DEFINE API "/custom_response"
    FOR get
        THEN {
            {
                status: 200,
                body: {
                    num: 1
                }
            };
        };
