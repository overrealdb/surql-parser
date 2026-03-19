-------- Query 3 (132.138µs) --------

[
	{
		id: person:jaime,
		name: 'Jaime',
		opts: {
			enabled: false,
			security: 'secure'
		},
		password: 'asdfgh'
	},
	{
		id: person:tobie,
		name: 'Tobie',
		opts: {
			enabled: true,
			security: 'secure'
		},
		password: '123456'
	}
]

-------- Query 4 (61.876µs) --------

[
	{
		id: person:jaime,
		name: 'Jaime',
		opts: {
			enabled: false
		}
	},
	{
		id: person:tobie,
		name: 'Tobie',
		opts: {
			enabled: true
		}
	}
]

-------- Query 5 (52.152µs) --------

[
	{
		id: person:jaime,
		name: 'Jaime',
		opts: {}
	},
	{
		id: person:tobie,
		name: 'Tobie',
		opts: {}
	}
]
