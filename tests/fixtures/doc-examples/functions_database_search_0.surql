CREATE book:1 SET title = "Rust Web Programming";
DEFINE ANALYZER book_analyzer TOKENIZERS blank, class, camel, punct FILTERS snowball(english);
DEFINE INDEX book_title ON book FIELDS title FULLTEXT ANALYZER book_analyzer BM25;
