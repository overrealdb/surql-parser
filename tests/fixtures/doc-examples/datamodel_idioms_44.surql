planet:earth
	.{1..2}
	.{
		name, 
		id,
-- Query with ->has->(?) on the current record
		contains: ->has->(?).@
	};
