DEFINE BUCKET my_bucket BACKEND "memory";

f"my_bucket:/lion_witch_wardrobe.txt".put("Once there were four children whose names were Peter, Susan...");
f"my_bucket:/other_book.txt".put("Um meine Geschichte zu erzählen, muß ich weit vorn anfangen.");
f"my_bucket:/other_book.txt".copy_if_not_exists("lion_witch_wardrobe.txt");
