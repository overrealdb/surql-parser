# SurrealQL for Zed

SurrealQL language support for the [Zed](https://zed.dev) editor.

## Features

- **Syntax highlighting** — powered by [tree-sitter-surrealql](https://github.com/Ce11an/tree-sitter-surrealql)
- **Code folding** — collapse blocks, functions, statements
- **Bracket matching** — `()`, `[]`, `{}`
- **Auto-indentation** — inside blocks and statements
- **Code outline** — DEFINE statements in the breadcrumb/sidebar

## LSP Support (optional)

For advanced features (completions, diagnostics, hover, formatting, signature help), install the `surql-lsp` language server:

```sh
# From the surql-parser repository
cargo install --path lsp

# Or when published to crates.io
cargo install surql-lsp
```

Then add to your Zed settings (`~/.config/zed/settings.json`):

```json
{
  "languages": {
    "SurrealQL": {
      "language_servers": ["surql-lsp"]
    }
  },
  "lsp": {
    "surql-lsp": {
      "binary": {
        "path": "surql-lsp"
      }
    }
  }
}
```

### LSP capabilities

| Feature | Description |
|---------|-------------|
| Diagnostics | Real-time error reporting with precise positions |
| Completions | Keywords, table names, field names, function signatures |
| Hover | Table info (fields, types) and function signatures |
| Formatting | Full document formatting |
| Signature help | Function parameter info while typing |
| Go-to-definition | Jump to DEFINE statements |

## Installation

### From Zed Extensions (when published)

1. Open Zed
2. `Cmd+Shift+X` → Search "SurrealQL" → Install

### Manual

1. Clone this directory
2. Symlink to Zed extensions:
   ```sh
   ln -s $(pwd) ~/.config/zed/extensions/installed/surrealql
   ```
3. Restart Zed

## Credits

- Tree-sitter grammar: [Ce11an/tree-sitter-surrealql](https://github.com/Ce11an/tree-sitter-surrealql) (MIT license)
- LSP server: [surql-parser](https://github.com/overrealdb/surql-parser)
