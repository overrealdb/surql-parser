/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a person for everyone in the array
FOR $name IN ['Tobie', 'Jaime'] {
	CREATE type::record('person', $name) CONTENT {
		name: $name
	};
};
