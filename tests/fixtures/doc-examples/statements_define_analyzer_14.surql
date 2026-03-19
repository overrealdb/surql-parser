DEFINE ANALYZER english_snowball TOKENIZERS class FILTERS snowball(english);
DEFINE ANALYZER german_snowball TOKENIZERS class FILTERS snowball(german);

RETURN [
    search::analyze("english_snowball", "Looking at some running cats"),
    search::analyze("german_snowball", "Sollen wir was trinken gehen?")
];
