/**[test]

[[test.results]]
value = "[{ id: person:one, name: 'Þor' }]"

[[test.results]]
value = "[{ id: dog:one, name: 'Fenrir' }]"

[[test.results]]
value = "[{ id: cat:one, name: 'Jólakötturinn' }]"

[[test.results]]
value = "[{ at: d'2025-10-08T05:00:18.418734Z', id: feeds:150g6q4l8lzscu7j57sj, in: person:one, out: cat:one }]"
skip-datetime = true 
skip-record-id-key = true

[[test.results]]
value = "[{ at: d'2025-10-08T05:00:18.420350Z', id: plays_with:itzsltgkyboieenc7rmz, in: dog:one, out: cat:one }]"
skip-datetime = true 
skip-record-id-key = true

[[test.results]]
value = "[{ "<-(SELECT * FROM feeds, plays_with ORDER BY at
)": [{ at: d'2025-10-08T05:00:18.418734Z', id: feeds:150g6q4l8lzscu7j57sj, in: person:one, out: cat:one }, { at: d'2025-10-08T05:00:18.420350Z', id: plays_with:itzsltgkyboieenc7rmz, in: dog:one, out: cat:one }] }]"
skip-datetime = true 
skip-record-id-key = true

[[test.results]]
value = "[{ "<-(SELECT * FROM ? ORDER BY at
)": [{ at: d'2025-10-08T05:00:18.418734Z', id: feeds:150g6q4l8lzscu7j57sj, in: person:one, out: cat:one }, { at: d'2025-10-08T05:00:18.420350Z', id: plays_with:itzsltgkyboieenc7rmz, in: dog:one, out: cat:one }] }]"
skip-datetime = true 
skip-record-id-key = true

*/

CREATE person:one SET name = "Þor";
CREATE dog:one SET name = "Fenrir";
CREATE cat:one SET name = "Jólakötturinn";
RELATE person:one->feeds->cat:one SET at = time::now();
RELATE dog:one->plays_with->cat:one SET at = time::now();

-- Select from both 'feeds' and 'plays_with'
SELECT <-(SELECT * FROM feeds, plays_with ORDER BY at) FROM cat:one;
-- Or any graph table
SELECT <-(SELECT * FROM ? ORDER BY at) FROM cat:one;
