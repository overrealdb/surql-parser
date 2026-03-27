SELECT
    name,
    ->contains->state AS states,
    ->contains->state->contains->city AS cities
FROM country:usa;
