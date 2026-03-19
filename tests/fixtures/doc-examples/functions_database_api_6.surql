DEFINE API "/body/json" FOR post
    MIDDLEWARE api::req::body("json")
    THEN {{ body: { parsed: $request.body } }};

DEFINE API "/body/cbor" FOR post
    MIDDLEWARE api::req::body("cbor")
    THEN {{ body: { parsed: $request.body } }};

DEFINE API "/body/plain" FOR post
    MIDDLEWARE api::req::body("plain")
    THEN {{ body: { parsed: $request.body } }};

DEFINE API "/body/bytes" FOR post
    MIDDLEWARE api::req::body("bytes")
    THEN {{ body: { parsed: $request.body } }};

DEFINE API "/body/native" FOR post
    MIDDLEWARE api::req::body("native")
    THEN {{ body: { parsed: $request.body } }};

DEFINE API "/body/auto" FOR post
    MIDDLEWARE api::req::body("auto")
    THEN {{ body: { parsed: $request.body } }};

api::invoke("/body/json", {
    method: "post",
    headers: { "content-type": "application/json" },
    body: <bytes>'{"name":"billy","billys_number":753}'
});

api::invoke("/body/cbor", {
    method: "post",
    headers: { "content-type": "application/cbor" },
    body: encoding::cbor::encode('CBOR!!')
});

api::invoke("/body/plain", {
    method: "post",
    headers: { "content-type": "text/plain" },
    body: <bytes>'plain text content'
});

api::invoke("/body/bytes", {
    method: "post",
    headers: { "content-type": "application/octet-stream" },
    body: <bytes>'raw bytes'
});

api::invoke("/body/native", {
    method: "post",
    headers: { "content-type": "application/vnd.surrealdb.native" },
    body: { native: "format" }
});

api::invoke("/body/auto", {
    method: "post",
    headers: { "content-type": "application/json" },
    body: <bytes>'{"auto":"json"}'
});

api::invoke("/body/auto", {
    method: "post",
    body: <bytes>'some data'
});
