/**[test]

[[test.results]]
value = "[{ has_been_friends: true, has_dated: { from: d'2020-12-25T00:00:00Z', to: d'2023-12-25T00:00:00Z' }, has_hated: { from: d'2023-12-25T00:00:00Z', to: d'2024-03-01T00:00:00Z' }, id: knows:dzj9fmhh22kodz8h2nhl, in: person:one, married_to: { since: d'2024-03-01T00:00:00Z' }, out: person:two, same_high_school: false }]"
skip-record-id-key = true

[[test.results]]
value = "[{ has_dated: { from: d'2019-09-10T00:00:00Z', to: d'2020-12-31T00:00:00Z' }, id: knows:gc5993blynpzf5l52qs4, in: person:one, out: person:three, same_high_school: true }]"
skip-record-id-key = true

*/

RELATE person:one->knows->person:two SET
	same_high_school = false,
    has_been_friends = true,
    has_dated = {
		from: d'2020-12-25',
		to: d'2023-12-25'
	},
    has_hated = {
		from: d'2023-12-25',
		to: d'2024-03-01'
	},
    married_to = {
		since: d'2024-03-01'
	};

RELATE person:one->knows->person:three SET
    same_high_school = true,
    has_dated = {
		from: d'2019-09-10',
		to: d'2020-12-31'
	};
