/**[test]

[[test.results]]
value = "[{ id: student:m1rdvvs0el37kiifume4, results: [{ date: '2017-06-18T08:00:00Z', name: 'Algorithmics', score: 76 }, { date: '2018-03-21T08:00:00Z', name: 'Concurrent Programming', score: 83 }, { date: '2018-09-17T08:00:00Z', name: 'Advanced Computer Science 101', score: 69 }, { date: '2019-04-20T08:00:00Z', name: 'Distributed Databases', score: 73 }] }]"
skip-record-id-key = true

*/

CREATE student SET results = [
	{ score: 76, date: "2017-06-18T08:00:00Z", name: "Algorithmics" },
	{ score: 83, date: "2018-03-21T08:00:00Z", name: "Concurrent Programming" },
	{ score: 69, date: "2018-09-17T08:00:00Z", name: "Advanced Computer Science 101" },
	{ score: 73, date: "2019-04-20T08:00:00Z", name: "Distributed Databases" },
];
