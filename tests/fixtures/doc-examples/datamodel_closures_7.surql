DEFINE TABLE test_table SCHEMAFULL;
DEFINE FIELD name ON test_table TYPE string;

DEFINE FUNCTION fn::test_create($name: string) -> object {
    CREATE test_table CONTENT { name: $name };
    { created: true, name: $name };
};

-- Function to create a record called for each name
SELECT VALUE fn::test_create($this) FROM ["Alice", "Bob", "Charlie"];
