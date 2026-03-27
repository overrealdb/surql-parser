DEFINE FUNCTION fn::start_timer($req: object, $next: function, $called_at: datetime) -> object {
    LET $res = $next($req);
    $res + { context: { called_at: $called_at }}
};

DEFINE FUNCTION fn::increment_num($req: object, $next: function) -> object {
    LET $res = $next($req);
    $res + { body: { num: $res.body.num + 1 } }
};

DEFINE API "/custom_response"
    FOR get
        MIDDLEWARE
            fn::start_timer(time::now()),
            fn::increment_num()
        THEN {
            {
                status: 200,
                body: {
                    num: 1
                }
            };
        };

api::invoke("/custom_response");
