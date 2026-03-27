/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Data retrieved from somewhere which contains many NONE values
LET $weather = [
	{
		city: 'London',
		temperature: 22.2,
		timestamp: 1722565566389
	},
	NONE,
	{
		city: 'London',
		temperature: 20.1,
		timestamp: 1722652002699
	},
    {
        city: 'Phoenix',
        temperature: 45.1,
        timestamp: 1722565642160
    },
    NONE,
    NONE,
    {
        city: 'Phoenix',
        temperature: 45.1,
        timestamp: 1722652070372
    },
];

FOR $data IN $weather {
    IF $data IS NONE {
        CONTINUE;
    };

	CREATE weather CONTENT $data;
};
