DEFINE ANALYZER lemme_english TOKENIZERS blank,class FILTERS lowercase,mapper('../tests/data/lemmatization-en.txt');

LET $text = "The Wheel of Time turns, and Ages come and pass, leaving memories that become legend. Legend fades to myth, and even myth is long forgotten when the Age that gave it birth comes again. In one Age, called the Third Age by some, an Age yet to come, an Age long past, a wind rose in the Mountains of Mist. The wind was not the beginning. There are neither beginnings nor endings to the turning of the Wheel of Time. But it was a beginning.";

LET $words = search::analyze("lemme_english", $text)
    .filter(|$c| $c.is_alpha());
FOR $word IN $words {
    UPSERT type::record("word", $word) SET frequency += 1;
};

SELECT * FROM word WHERE frequency >=3 ORDER BY frequency DESC;
