# surql-macros

Compile-time SurrealQL validation macros for Rust.

## Macros

### `surql_check!`

Validates SurrealQL syntax at compile time. Returns `&'static str`.

```rust
let query = surql_check!("SELECT * FROM user WHERE age > 18");
```

Compile error if the query is invalid:
```rust
let query = surql_check!("SELEC * FORM user"); // compile error
```

### `surql_query!`

Validates SurrealQL syntax AND verifies `$param` placeholders match at compile time.

```rust
let sql = surql_query!(
    "SELECT * FROM user WHERE age > $min AND name = $name",
    min,
    name
);
```

Compile errors for missing or extra parameters:
```rust
// error: missing parameter $name
let sql = surql_query!("SELECT * FROM user WHERE name = $name", );

// error: extra parameter city
let sql = surql_query!("SELECT * FROM user WHERE age > $min", min, city);
```

### `surql_query!` with schema-aware type checking

With `schema = "surql/"`, validates parameter types against DEFINE FUNCTION signatures:

```rust
let sql = surql_query!(
    "SELECT * FROM agent WHERE role = $role AND active = $active",
    role: String,
    active: bool,
    schema = "surql/"
);
```

### `#[surql_function]`

Validates function name, parameter count, and types at compile time.

```rust
// Basic: validates name only
#[surql_function("fn::get_user")]
pub fn get_user_call(id: &str) -> String {
    format!("fn::get_user('{id}')")
}

// With schema: validates arity + types against DEFINE FUNCTION in .surql files
#[surql_function("fn::migration::apply", schema = "surql/")]
pub fn migration_apply(mig_id: &str, agent_id: &str) -> String {
    format!("fn::migration::apply({mig_id}, {agent_id})")
}
```

Type mapping: `string`→`&str/String`, `int`→`i64/i32`, `float`→`f64`, `bool`→`bool`, `record<T>`→`&str/String/RecordId`, `array<T>`→`Vec`, `option<T>`→`Option`

## Build-time Codegen

The `surql-parser` crate (with `build` feature) provides `build.rs` helpers:

```rust
// build.rs
surql_parser::build::validate_schema("surql/");
surql_parser::build::generate_typed_functions("surql/", format!("{out_dir}/surql_functions.rs"));
```

This generates typed constants:
```rust
/// SurrealQL function: `fn::greet`
/// Parameters: `$name: string`
pub const FN_GREET: &str = "fn::greet";
```

## Requirements

- SurrealDB 3.x+ syntax
- Rust edition 2024
