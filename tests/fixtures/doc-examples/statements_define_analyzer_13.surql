DEFINE ANALYZER example_ngram TOKENIZERS class FILTERS ngram(1,3);
search::analyze("example_ngram", "apple banana");
