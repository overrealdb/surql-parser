-- Use the .values() method to turn each record into
-- an array of values, then return all inside an array
SELECT @.{
    name,
    id,
    orbits: ->orbits->star
}.values()
    FROM planet;

-- Grab all records first, then use .map() to convert
-- each one into an array of values
(SELECT 
    name,
    id,
    ->orbits->star AS orbits
FROM planet)
    .map(|$obj| $obj.values());
