api::invoke("/test");

api::invoke("/test", {
    body: {
       hi: "please",
        give: "me",
        the: "information"
    }
});
