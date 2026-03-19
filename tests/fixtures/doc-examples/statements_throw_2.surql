/**[test]

[[test.results]]
value = "[{ id: event:one, time: d'2025-10-08T07:15:04.994633Z' }]"

[[test.results]]
value = "[{ id: event:two, time: d'2025-10-08T07:15:04.996995Z' }]"

[[test.results]]
error = ""An error occurred: [{ id: event:one, time: d'2025-10-08T07:15:04.994633Z' }, { id: event:two, time: d'2025-10-08T07:15:04.996995Z' }]""

*/

CREATE event:one SET time = d'2025-10-08T07:15:04.994633Z';
CREATE event:two SET time = d'2025-10-08T07:15:04.996995Z';
THROW SELECT * FROM event;
