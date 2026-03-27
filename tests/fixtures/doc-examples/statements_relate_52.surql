SELECT
    id,
	-- access id and population on a single line
    ->contains->state.{id, population},
    ->contains->state->contains->city.id
FROM country:usa;
