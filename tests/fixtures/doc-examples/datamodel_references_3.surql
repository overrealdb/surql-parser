DEFINE FIELD comics ON person TYPE option<array<record<comic_book>>> REFERENCE;
DEFINE FIELD borrowed_comics ON person TYPE option<array<record<comic_book>>> REFERENCE;
DEFINE FIELD owned_by ON comic_book COMPUTED <~(person FIELD comics);
DEFINE FIELD borrowed_by ON comic_book COMPUTED <~(person FIELD borrowed_comics);
DEFINE FIELD all_readers ON comic_book COMPUTED <~(person FIELD comics borrowed_comics);

CREATE person:one SET comics = [comic_book:one];
CREATE person:two SET borrowed_comics = [comic_book:one];
CREATE comic_book:one SET title = "Loki, God of Stories";
SELECT * FROM comic_book;
