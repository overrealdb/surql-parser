-- Query attempting to create half a million `person` records
CREATE |person:500000| SET age = 46, username = "john-smith" TIMEOUT 500ms;
