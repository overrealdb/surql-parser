DEFINE ANALYZER lemme_english TOKENIZERS blank,class FILTERS lowercase,mapper('../tests/data/lemmatization-en.txt');

RETURN [
    search::analyze("lemme_english", "He drove and swam"),
];
