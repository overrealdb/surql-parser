FOR $data IN [
	{
		name: 'Billy',
		num: 9
	},
    {
		name: 'Bobby',
		num: 10
	},
].map(|$o| {
    id: $o.num,
    name: $o.name
}) {
    UPSERT type::record('person', $data.id) CONTENT $data;
};
