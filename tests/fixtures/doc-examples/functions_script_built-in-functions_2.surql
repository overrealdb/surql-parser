LET $something = 123;
LET $obj = {
	nested: 456
};

LET $arr = [
	{ value: 1 },
	{ value: 2 },
	{ value: 3 },
	{ value: 4 },
	{ value: 5 },
	{ value: 6 },
];

RETURN function() {
	// Get the value for a variable
	const something = await surrealdb.value("$something");

	// Get the value for a nested property
	const nested = await surrealdb.value("$obj.nested");

	// Filter properties from an array
	const fromArray = await surrealdb.value("$arr[WHERE value > 3].value");

	return { something, nested, fromArray };
}
