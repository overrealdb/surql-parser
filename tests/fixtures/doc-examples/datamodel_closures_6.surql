-- 1. Create a test table and function
DEFINE TABLE test_table SCHEMAFULL;
DEFINE FIELD name ON test_table TYPE string;

DEFINE FUNCTION fn::test_create($name: string) -> object {
    CREATE test_table CONTENT { name: $name };
    { created: true, name: $name };
};

-- 2. Call the function directly - works
fn::test_create("direct_call");

-- 3. Call the function inside .map() - fails
LET $names = ["Alice", "Bob", "Charlie"];
$names.map(|$n| fn::test_create($n));
-- Error: "Couldn't write to a read only transaction"
