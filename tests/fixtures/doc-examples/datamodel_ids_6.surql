/**[test]

[[test.results]]
value = "[{ celsius: 37.5f, id: temperature:17493, time: d'2025-10-03T01:09:50.155406Z' }]"

[[test.results]]
value = "[{ events: ['Galactic senate convenes', 'Mr. Bean still waits in a field'], id: year:⟨29878977097987987979232⟩ }]"

*/

CREATE temperature:17493 SET time = time::now(), celsius = 37.5;
CREATE year:29878977097987987979232 SET
    events = [
        "Galactic senate convenes",
        "Mr. Bean still waits in a field"
    ];
