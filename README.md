<p align="center">
  <img src="assets/logo.svg" alt="surql-parser" width="120" />
</p>

<h1 align="center">surql-parser</h1>

<p align="center">
  Standalone SurrealQL parser extracted from <a href="https://surrealdb.com">SurrealDB</a>.<br/>
  Parse SurrealQL queries into an AST without depending on the SurrealDB engine.
</p>

<p align="center">
  <a href="https://github.com/overrealdb/surql-parser/actions/workflows/ci.yml"><img src="https://github.com/overrealdb/surql-parser/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/overrealdb/surql-parser/actions/workflows/security.yml"><img src="https://github.com/overrealdb/surql-parser/actions/workflows/security.yml/badge.svg" alt="Security" /></a>
  <a href="https://crates.io/crates/surql-parser"><img src="https://img.shields.io/crates/v/surql-parser.svg" alt="crates.io" /></a>
  <a href="https://docs.rs/surql-parser"><img src="https://docs.rs/surql-parser/badge.svg" alt="docs.rs" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License" /></a>
</p>

## Use Cases

- Migration tools and schema analyzers
- Linters and formatters for `.surql` files
- IDE extensions (Zed, VS Code) with syntax analysis
- Code generation from SurrealQL definitions
- CI validation of SurrealQL files

## Quick Start

```rust
use surql_parser::parse;

let ast = parse("SELECT name, age FROM user WHERE age > 18 ORDER BY name").unwrap();
assert_eq!(ast.expressions.len(), 1);
```

### Parse DDL

```rust
let ast = surql_parser::parse(
    "DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; }"
).unwrap();
```

### Parse type annotations

```rust
let kind = surql_parser::parse_kind("option<record<user>>").unwrap();
```

### Extract schema definitions

```rust
let defs = surql_parser::extract_definitions("
    DEFINE TABLE user SCHEMAFULL;
    DEFINE FIELD name ON user TYPE string;
    DEFINE FIELD age ON user TYPE int DEFAULT 0;
    DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
    DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
").unwrap();

assert_eq!(defs.tables.len(), 1);
assert_eq!(defs.fields.len(), 2);
assert_eq!(defs.indexes.len(), 1);
assert_eq!(defs.functions.len(), 1);
```

### Check reserved keywords

```rust
assert!(surql_parser::is_reserved_keyword("SELECT"));
assert!(!surql_parser::is_reserved_keyword("username"));
```

## CLI Tool

```sh
cargo install surql-parser --features cli

surql check schema/**/*.surql      # validate .surql files
surql schema schema/               # extract full schema
surql fmt file.surql               # format SurrealQL
surql functions schema/            # list fn::* definitions
surql tables schema/               # list table definitions
```

## Testing

```sh
cargo test                               # parser only (instant)
cargo test --features validate-mem       # + in-memory SurrealDB
cargo test --features validate-docker    # + real SurrealDB in Docker
```

Validation deps (surrealdb, testcontainers) are dev-dependencies only — they don't leak to library consumers.

## SurrealDB Compatibility

| surql-parser | SurrealDB | Status |
|-------------|-----------|--------|
| 0.1.x       | 3.x       | Active |

Parser source is auto-synced from SurrealDB via an automated pipeline. See [UPSTREAM_SYNC.md](UPSTREAM_SYNC.md) for details.

## How It Works

The parser source code is **extracted from SurrealDB** using an AST-level Rust transformer (`tools/transform/`). This ensures 100% compatibility with SurrealDB's parser while removing engine-specific execution code.

The sync pipeline:
1. Copies `syn/` (lexer, parser) and `sql/` (AST types) from SurrealDB source
2. Rewrites imports via declarative rules (`mappings.toml`)
3. Strips execution-layer code (318 impl blocks removed automatically)
4. Validates compilation

## License

Apache 2.0 — same as SurrealDB.
