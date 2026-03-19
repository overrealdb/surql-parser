/**[test]

[[test.results]]
value = "[{ id: article:10 }]"

[[test.results]]
value = "[{ id: article:⟨10⟩ }]"

[[test.results]]
value = "[{ id: article:article10 }]"

[[test.results]]
value = "[article:10, article:⟨10⟩, article:article10]"

*/

CREATE article SET id = 10;
CREATE article SET id = "10";
CREATE article SET id = "article10";
SELECT VALUE id FROM article;
