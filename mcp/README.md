# surql-mcp — SurrealQL Playground via MCP

MCP (Model Context Protocol) server with an embedded SurrealDB instance. Provides an interactive playground for executing SurrealQL queries, loading schema files, and exploring database state.

## Install

```bash
cargo install --path mcp
```

## Connect

### Claude Code
```bash
claude mcp add surql-mcp -- surql-mcp
```

### Zed (via extension)
Install the SurrealQL extension — it registers `surql-mcp` automatically.

### Manual (settings.json)
```json
{
  "mcpServers": {
    "surql-mcp": { "command": "surql-mcp" }
  }
}
```

## Tools

| Tool | Description |
|------|-------------|
| `exec` | Execute a SurrealQL query, returns JSON result |
| `load_project` | Load .surql files (schema → functions → migrations → examples, auto-OVERWRITE) |
| `load_file` | Execute a single .surql file |
| `schema` | Show current database schema (INFO FOR DB) |
| `describe` | Show detailed info for a specific table |
| `reset` | Clear the database |
| `manifest` | Read overshift manifest.toml and show project config |
| `load_manifest` | Load an overshift project (schema modules + migrations) |
| `compare` | Compare playground schema with expected JSON (SchemaDiff) |
| `verify` | Shadow DB verification (`verify_only` for read-only mode) |
| `check` | Validate .surql files (file or directory, recursive) |
| `rollback` | Rollback playground to a target migration version |
| `graph_affected` | Show which tables break if a table is dropped (reverse record<> links) |
| `graph_traverse` | N-hop graph traversal from a table (forward or reverse) |
| `graph_siblings` | Find tables sharing record<> link targets |

### load_project priority

Files are loaded in this order:
1. `schema*` — schema definitions
2. `function*` — function definitions
3. `migrations/` — schema migrations
4. Other files
5. `examples/`, `seed/`, `test/` — last

Set `clean: false` to skip the automatic database reset before loading.

## Examples

```
> exec: CREATE user:alice SET name = 'Alice', age = 30
→ [{ id: user:alice, name: "Alice", age: 30 }]

> load_project: path = "./surql"
→ Loaded 4/4 files from `./surql` (clean)

> schema
→ { tables: { user: "DEFINE TABLE user SCHEMAFULL ...", ... } }

> describe: table = "user"
→ { fields: { name: "string", age: "int", ... } }

> reset
→ Database cleared
```
