{
	events: {},
	fields: {
		friends: 'DEFINE FIELD friends ON person TYPE array<record<person>> PERMISSIONS FULL',
		"friends.*": 'DEFINE FIELD friends.* ON person TYPE record<person> PERMISSIONS FULL'
	},
	indexes: {},
	lives: {},
	tables: {}
};
