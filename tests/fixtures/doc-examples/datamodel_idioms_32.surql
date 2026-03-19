CREATE
	// One planet
	planet:earth,
	// Two countries
	country:us, country:canada,
	// Four states/provinces
	state:california, state:texas,
	province:ontario, province:bc,
	// Eight cities
	city:los_angeles, city:san_francisco,
	city:houston,     city:dallas,
	city:vancouver,   city:victoria,
	city:toronto,     city:ottawa
		// Give them each names like 'earth', 'us', 'bc', etc.
	SET name = id.id();

// Record and graph links from planet to country
UPDATE planet:earth     SET next = [country:us, country:canada];
RELATE planet:earth     ->has->    [country:us, country:canada];

// Record and graph links from country to state/province
UPDATE country:us       SET next = [state:california, state:texas];
UPDATE country:canada   SET next = [province:ontario, province:bc];
RELATE country:us       ->has->    [state:california, state:texas];
RELATE country:canada   ->has->    [province:bc, province:ontario];

// Record and graph links from state/province to city
UPDATE state:california SET next = [city:los_angeles, city:san_francisco];
UPDATE state:texas      SET next = [city:houston, city:dallas];
UPDATE province:ontario SET next = [city:toronto, city:ottawa];
UPDATE province:bc      SET next = [city:vancouver, city:victoria];
RELATE state:california ->has->    [city:los_angeles, city:san_francisco];
RELATE state:texas      ->has->    [city:houston, city:dallas];
RELATE province:bc      ->has->    [city:vancouver, city:victoria];
RELATE province:ontario ->has->    [city:toronto, city:ottawa];
