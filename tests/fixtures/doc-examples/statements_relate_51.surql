SELECT
    id,
    ->contains->state.id,
    ->contains->state.population,
    ->contains->state->contains->city.id
FROM country:usa;
