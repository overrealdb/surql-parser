/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: book:demian, language: 'German', title: 'Demian. Die Geschichte von Emil Sinclairs Jugend' }]"

[[test.results]]
value = "[{ id: author:hesse, name: 'Hermann Hesse' }]"

[[test.results]]
error = ""Found book:demian for field `out`, with record `wrote:l4xjcgqkgm7vmqqt4iah`, but field must conform to: $value.language = 'English'""

*/

DEFINE FIELD out ON TABLE wrote TYPE record<book> ASSERT $value.language = "English";

CREATE book:demian SET title = "Demian. Die Geschichte von Emil Sinclairs Jugend", language = "German";
CREATE author:hesse SET name = "Hermann Hesse";

RELATE author:hesse->wrote->book:demian;
