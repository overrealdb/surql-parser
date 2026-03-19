SELECT 
	name, 
	->(wrote WHERE written_at = "Athens")->book.{ name, id } AS books_written_in_athens
FROM person;
