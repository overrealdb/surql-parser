IF f"my_bucket:/my_book.txt".exists() {
    THROW "Whoops, already there!"
} ELSE {
    f"my_bucket:/my_book.txt".put("Some content")
};
