api::invoke("/test", {
    body: <bytes> '{ "a": true }',
    headers: {
        "Content-Type": "application/json",
        Accept: "application/cbor",
    }
});
