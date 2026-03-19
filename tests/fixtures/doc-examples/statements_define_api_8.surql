DEFINE FUNCTION fn::increment_num($req: object, $next: function) -> object {
    LET $res = $next($req);
    $res + { body: { num: $res.body.num + 1 } }
};

DEFINE API "/custom_response"
    FOR get
        MIDDLEWARE
            fn::increment_num()
        THEN {
            {
                status: 200,
                body: {
                    num: 1
                }
            };
        };
