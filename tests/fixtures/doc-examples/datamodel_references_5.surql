{
	events: {},
	fields: {
		comics: 'DEFINE FIELD comics ON person TYPE none | array<record<comic_book>> REFERENCE ON DELETE IGNORE PERMISSIONS FULL',
		"comics.*": 'DEFINE FIELD comics.* ON person TYPE record<comic_book> REFERENCE ON DELETE IGNORE PERMISSIONS FULL'
	},
	indexes: {},
	lives: {},
	tables: {}
}
