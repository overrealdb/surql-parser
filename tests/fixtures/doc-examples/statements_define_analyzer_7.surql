DEFINE ANALYZER example_uppercase TOKENIZERS class FILTERS uppercase;
search::analyze("example_uppercase", "Hello World");
