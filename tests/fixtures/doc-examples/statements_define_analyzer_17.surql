-- Creates a simple analyzer removing diacritics marks
DEFINE ANALYZER ascii TOKENIZERS class FILTERS lowercase,ascii;
