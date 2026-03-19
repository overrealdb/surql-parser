-- Update certain fields on all records
UPDATE person MERGE {
	settings: {
		marketing: true,
	},
};

-- Update certain fields on a specific record
UPDATE person:tobie MERGE {
	settings: {
		marketing: true,
	},
};
