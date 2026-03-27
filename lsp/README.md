# surql-lsp

SurrealQL Language Server Protocol implementation. Provides IDE features for `.surql` files.

## Features

- **Diagnostics** -- parse errors with recovery, undefined table warnings, record link validation
- **Completions** -- context-aware: tables first after FROM, keywords with trailing space, field names, functions, builtins, params
- **Hover** -- 255/255 keyword coverage, table schema with fields/types, nested object fields (`settings.theme`), graph paths (`->manages->project.name`), doc links to surrealdb.com
- **Go to Definition** -- jump to DEFINE TABLE/FIELD/FUNCTION/INDEX source across files
- **Find References** -- tables and fields (scoped by table context), functions
- **Rename (F2)** -- cross-file table rename (FROM, ON, record<>, :id patterns)
- **Code Actions** -- quick fix: suppress warnings with `-- surql-allow: undefined-table`
- **Signature Help** -- function parameter hints with types
- **Document Symbols** -- breadcrumb outline of DEFINE statements
- **Formatting** -- configurable via `.surqlformat.toml` (10 options)
- **Semantic Tokens** -- keyword, function, variable, string, number, operator, type, comment
- **Progress Notifications** -- status bar shows "SurrealQL: Scanning..." during rebuild
- **Incremental Rebuild** -- only re-parses the saved file, not the entire workspace
- **Code Lens** -- inline schema info
- **Embedded SurrealQL** -- validates `surql_query!()` and `surql_check!()` in Rust files
- **NS/DB scoping** -- USE statements → manifest.toml → .env (.env.local/.env.development) → default
- **Monorepo detection** -- warns when multiple SurrealDB projects found in workspace
- **8 Slash Commands** -- /surql-schema, /surql-relations, /surql-info, /surql-check, /surql-migrations, /surql-docs, /surql-graph, /surql-dependents

## Installation

```bash
cargo install --path lsp --force
```

For embedded DB validation (optional):
```bash
cargo install --path lsp --force --features embedded-db
```

## Formatter Configuration

Create `.surqlformat.toml` in your project root:

```toml
uppercase_keywords = true       # SELECT, FROM, WHERE (default: true)
indent_style = "tab"            # "tab" or "space" (default: "tab")
indent_width = 4                # spaces per indent level (default: 4)
newline_after_semicolon = false  # break after ; (default: false)
newline_before_where = false     # WHERE on new line (default: false)
newline_before_set = false       # SET on new line (default: false)
newline_before_from = false      # FROM on new line (default: false)
trailing_semicolon = false       # add ; at end of file (default: false)
collapse_blank_lines = false     # reduce consecutive blank lines (default: false)
max_blank_lines = 2              # max blank lines when collapsing (default: 2)
```

All options default to off except `uppercase_keywords`.

## Editor Setup

### Zed

The Zed extension discovers `surql-lsp` from PATH automatically. See `editors/zed/`.

### VS Code

Configure `surrealql.lspPath` in settings or ensure `surql-lsp` is in PATH. See `editors/vscode/`.

## Architecture

- `server.rs` -- LSP backend (`tower-lsp`), schema graph management, file manifest generation
- `completion.rs` -- context-aware completion (table context detection from cursor position)
- `context.rs` -- DML table reference extraction from lexer tokens
- `diagnostics.rs` -- error-recovering parser diagnostics
- `formatting.rs` -- configurable lexer-based formatter
- `embedded.rs` -- Rust macro extraction (`surql_query!`, `surql_check!`)
- `embedded_db.rs` -- optional dual in-memory SurrealDB (workspace + meta)
- `signature.rs` -- function signature help from schema graph
- `keywords.rs` -- keyword list (synced with parser)
