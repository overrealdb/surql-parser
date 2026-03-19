/**[test]

[[test.results]]
value = "[{ id: star:sun, name: 'The Sun' }]"

[[test.results]]
value = "[{ id: planet:earth, name: 'Earth' }]"

[[test.results]]
value = "[{ id: planet:earth, name: 'Earth', orbits: [star:sun] }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: planet:earth, name: 'Earth', orbits: [star:sun] }]"

[[test.results]]
value = "[{ id: planet:earth, name: 'Earth', orbits: [star:sun] }]"

*/

, [{ id: planet:earth, name: 'Earth', orbits: [star:sun] }]]
CREATE star:sun SET name = "The Sun";
CREATE planet:earth SET name = "Earth";
RELATE planet:earth->orbits->star:sun;

-- Regular SELECT query
SELECT 
    name,
    id,
    ->orbits->star AS orbits
FROM planet;

-- SELECT query using `@` and destructuring
SELECT @.{
    name,
    id,
    orbits: ->orbits->star
} FROM planet;
