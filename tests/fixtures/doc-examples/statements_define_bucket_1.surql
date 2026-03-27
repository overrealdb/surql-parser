-- Create a file by adding some content
f"my_bucket:/my_book.txt".put("Once there were four children whose names were Peter, Susan, Edmund, and Lucy.");
-- Copy it to a new file name
f"my_bucket:/my_book.txt".copy("lion_witch_wardrobe.txt");
-- Read the file as bytes
f"my_bucket:/lion_witch_wardrobe.txt".get();
-- Cast the bytes to a string
<string>f"my_bucket:/lion_witch_wardrobe.txt".get();
