DEFINE ANALYZER example_lowercase TOKENIZERS class FILTERS lowercase;
search::analyze("example_lowercase", "Hello World");
