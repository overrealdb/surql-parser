FOR $data IN [
	{
		id: 9,
		name: 'Billy'
	},
	{
		id: 10,
		name: 'Bobby'
	}
] {
	UPSERT type::record('person', $data.id) CONTENT $data;
};
