SELECT @.{..}.{ id, likes: ->likes->person.@ } FROM person TIMEOUT 1s;
