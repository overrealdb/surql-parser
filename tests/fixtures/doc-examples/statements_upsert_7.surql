/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ class: 'wizard', id: user:billy, metadata: { likes: ['strawberries'] }, name: 'Billy', official_name: 'Billy the wizard' }]"

[[test.results]]
value = ""Database index `unique_key` already contains ['Billy', 'wizard'], with record `user:billy`""

*/

DEFINE INDEX unique_key ON TABLE user FIELDS name, class UNIQUE;
DEFINE FIELD official_name ON TABLE user VALUE name + " the " + class;
CREATE user:billy SET name = "Billy", class = "wizard", metadata = { likes: ["strawberries"] };
CREATE user:billy_der_zweite SET name = "Billy", class = "wizard", metadata = { likes: ["strawberries", "fields"] };
