-- Original query
planet:earth.{1..2}.{ name, id, contains: ->has->(?).@ };

-- Unfolds to:
planet:earth
	.{
		name, 
		id,
		contains: ->has->(?).{
		  name, 
		  id,
		  contains: ->has->(?)
	    }
	};
