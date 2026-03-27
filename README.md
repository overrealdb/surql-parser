<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="icon-512.png" width="120" />
    <source media="(prefers-color-scheme: light)" srcset="icon-light-512.png" width="120" />
    <img src="icon-512.png" alt="surql-parser" width="120" />
  </picture>
</p>

<h1 align="center">surql-parser</h1>

<p align="center">
  Standalone SurrealQL parser extracted from <a href="https://surrealdb.com">SurrealDB</a>.<br/>
  Parse SurrealQL queries into an AST without depending on the SurrealDB engine.
</p>

<p align="center">
  <a href="https://github.com/overrealdb/surql-parser/actions/workflows/ci.yml"><img src="https://github.com/overrealdb/surql-parser/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/overrealdb/surql-parser/actions/workflows/security.yml"><img src="https://github.com/overrealdb/surql-parser/actions/workflows/security.yml/badge.svg" alt="Security" /></a>
  <a href="https://codecov.io/gh/overrealdb/surql-parser"><img src="https://codecov.io/gh/overrealdb/surql-parser/graph/badge.svg" alt="codecov" /></a>
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

## Compile-Time Tools

### `surql-macros` — proc-macro crate

Validate SurrealQL at compile time:

```toml
[dependencies]
surql-macros = "0.1"
```

```rust
use surql_macros::{surql_check, surql_function};

// Compile-time validated query — typo here is a compile error
const QUERY: &str = surql_check!("SELECT * FROM user WHERE age > 18");

// Compile-time validated function name
#[surql_function("fn::get_user")]
fn get_user_call(id: &str) -> String {
    format!("fn::get_user('{id}')")
}
```

Errors show both the SurrealQL location and the Rust source location:

```
error: Invalid SurrealQL: Unexpected token `WHERE`, expected FROM
 --> [1:10]
  |
1 | SELECT * WHERE age > 18
  |          ^^^^^
```

### `build.rs` helper (feature `build`)

Validate `.surql` files and generate typed constants at build time:

```toml
[build-dependencies]
surql-parser = { version = "0.1", features = ["build"] }
```

```rust
// build.rs
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    surql_parser::build::validate_schema("surql/");
    surql_parser::build::generate_typed_functions(
        "surql/",
        format!("{out_dir}/surql_functions.rs"),
    );
}
```

This generates constants like `FN_GET_USER: &str = "fn::get_user"` with doc comments showing parameter types and return types. See `examples/sample-project/` for a complete working example.

## CLI Tool

```sh
cargo install surql-parser --features cli

surql check schema/**/*.surql      # validate .surql files
surql fmt file.surql               # format SurrealQL
surql info schema/                 # show schema summary
surql diff schema/                 # show uncommitted schema changes
surql docs schema/                 # generate markdown docs
surql lint schema/                 # run SurrealQL-specific lints
surql test tests/                  # run .surql test files
```

## Testing

```sh
cargo test                               # parser tests (instant)
cargo test --features build              # + build helper tests
cargo test -p surql-macros               # proc-macro tests (trybuild)
cargo test -p surql-sample-project       # e2e: build.rs + macros
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

## Workspace

This repository is a Rust workspace with several crates:

| Crate | Description |
|-------|-------------|
| [`surql-parser`](.) | Core parser — AST, schema graph, formatting, linting, diff |
| [`surql-macros`](macros/) | Compile-time validation with schema-aware type checking |
| [`surql-lsp`](lsp/) | Language Server — hover, completions, rename, diagnostics, 8 slash commands |
| [`surql-mcp`](mcp/) | MCP playground — 15 tools (query, graph, verify, rollback) |
| [`overshift`](overshift/) | Migration engine — schema modules, rollback, shadow DB verification |
| [`surql` CLI](src/bin/surql.rs) | `check`, `fmt`, `info`, `diff`, `docs`, `lint`, `test` |
| [Zed extension](editors/zed/) | Syntax highlighting, 8 slash commands, MCP context server |
| [VS Code extension](editors/vscode/) | TextMate grammar, LSP client |

## License

Apache 2.0 — same as SurrealDB.
