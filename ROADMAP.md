# SurrealQL Tooling Roadmap

## Current Sprint — Status

### Done
- [x] 421 builtin functions with docs from SurrealDB
- [x] Hover: tables, fields, functions, builtins, keywords, types
- [x] Goto-definition: tables, functions, fields
- [x] Find All References
- [x] Signature help: user fn:: and builtins
- [x] Namespace-aware completions (string::, array::, etc.)
- [x] Snippet expansion with parameter names
- [x] Resilient workspace schema (recovery parser per-file)
- [x] Schema merge: SCHEMAFULL wins, dedup fields/indexes
- [x] Document schema overlay (live schema from unsaved files)
- [x] Embedded SurrealQL in Rust (hover, diagnostics in surql_query!/surql_check!)
- [x] Tree-sitter grammar (our own, Apache-2.0, SDB3+)
- [x] Keyword documentation with docs links
- [x] Type documentation (option, record, array, etc.)
- [x] Docs URLs in builtin hover
- [x] Build helpers: non-crashing validate_schema, per-file generate_typed_functions
- [x] COMMENT clause support in hover
- [x] Record ID parsing (user:alice → hover on table user)
- [x] Hover range (cursor move → hover updates)
- [x] 310+ tests

### Remaining (before SurrealDB integration)
- [ ] **Document Symbols** — outline panel in Zed (TABLE → FIELD → INDEX hierarchy)
- [ ] **Code Lens** — inline annotations above DEFINE TABLE ("4 fields · 2 indexes")
- [ ] **Context-aware field hover** — detect table from FROM/UPDATE/SET context
- [ ] **Doc examples as test fixtures** — extract from SurrealDB docs, commit as fixtures
- [ ] **Performance baseline** — HashMap index for fields, benchmark on large schemas
- [ ] **DEFINE API support** — grammar + LSP for SDB3 REST API definitions

---

## Next Sprint — SurrealDB Embedded Engine

### Overview
Two LSP binary variants:
- **`surql-lsp`** (lite, ~9 MB) — parser-only, all current features
- **`surql-lsp-full`** (~60 MB) — includes embedded SurrealDB in-memory engine

### Architecture
```
surql-lsp-full (Rust binary)
  ├─ surql-parser (parsing, schema extraction)
  └─ surrealdb crate (kv-mem)
       └─ In-memory SurrealDB instance
            ├─ Applies workspace .surql files as migrations (ordered)
            ├─ INFO FOR DB → real schema introspection
            ├─ Execute arbitrary queries
            └─ Validates runtime behavior (not just syntax)
```

### Cargo.toml
```toml
[features]
default = []
embedded-db = ["dep:surrealdb"]

[dependencies]
surrealdb = { version = "3", default-features = false, features = ["kv-mem"], optional = true }
```

### Two binaries
```toml
[[bin]]
name = "surql-lsp"
path = "src/main.rs"

[[bin]]
name = "surql-lsp-full"
path = "src/main_full.rs"
required-features = ["embedded-db"]
```

### CI: Cross-platform builds
GitHub Actions matrix:
| Platform | Architecture | Asset name |
|----------|-------------|------------|
| macOS    | arm64       | surql-lsp-darwin-aarch64.tar.gz |
| macOS    | x86_64      | surql-lsp-darwin-x86_64.tar.gz |
| Linux    | x86_64      | surql-lsp-linux-x86_64.tar.gz |
| Linux    | arm64       | surql-lsp-linux-aarch64.tar.gz |
| Windows  | x86_64      | surql-lsp-windows-x86_64.zip |

Same matrix for `-full` variants.

### Zed Extension: Auto-download
Extension's `language_server_command()`:
1. Check `worktree.which("surql-lsp")` — user-installed binary
2. If not found → `zed::latest_github_release("overrealdb/surql-parser")`
3. Detect platform: `zed::current_platform()` → (Os, Architecture)
4. Download matching asset to extension cache
5. Settings choose lite vs full variant

### Features enabled by embedded DB
| Feature | Lite | Full |
|---------|------|------|
| Syntax highlighting | ✅ | ✅ |
| Hover/completions/goto-def | ✅ | ✅ |
| Parse-time diagnostics | ✅ | ✅ |
| **Runtime diagnostics** (undefined table/field) | ❌ | ✅ |
| **Execute query** (code action → show results) | ❌ | ✅ |
| **Validate migration** (dry-run in-memory) | ❌ | ✅ |
| **Real schema introspection** (INFO FOR DB) | ❌ | ✅ |
| **Try-out panel** (run arbitrary SurrealQL) | ❌ | ✅ |

### WASM Note
SurrealDB's `ring` crate dependency blocks compilation to `wasm32-wasip2`.
Cannot embed SurrealDB directly in Zed WASM extension.
LSP binary approach is the correct architecture — same pattern as rust-analyzer, buf, protols.

---

## ERD / Schema Visualization

### Document Symbols (immediate)
LSP `textDocument/documentSymbol` → outline panel in Zed sidebar:
```
TABLE user (SCHEMAFULL) — Main user table
  FIELD name : string
  FIELD email : string — Primary email
  INDEX email_idx (UNIQUE)
  EVENT user_created
TABLE post (SCHEMAFULL)
  FIELD title : string
  FIELD author : record<user>
```

### Code Lens (next)
Inline annotations above DEFINE TABLE:
```
4 fields · 2 indexes · →post · ←comment
DEFINE TABLE user SCHEMAFULL;
```

### CLI ERD (future)
`surql schema-erd --format html --open`
- Interactive D3.js graph in browser
- Tables as nodes, record<> links as edges
- Clickable: jumps to file:// locations

---

## Multi-IDE Support (future)

### Neovim
- Tree-sitter: `:TSInstall surrealql` (publish to nvim-treesitter)
- LSP: configure in lspconfig, binary from GitHub Releases
- Injections: `after/queries/rust/injections.scm` for surql_query!

### VS Code
- Extension with vscode-languageclient
- TextMate grammar (convert from tree-sitter)
- Marketplace publishing

### Helix
- Tree-sitter in languages.toml
- LSP in languages.toml

---

## Research Notes

### Zed Extension API (2026-03)
- ✅ Language server, tree-sitter grammars, slash commands, process spawn, HTTP client
- ✅ GitHub Releases download (auto-install LSP binary)
- ✅ Platform detection (Os, Architecture)
- ❌ Webview, custom panels, cross-language injection (issue #8795)
- ❌ Semantic tokens from secondary LSP

### SurrealDB COMMENT Support
All DEFINE statements support `COMMENT 'text'` — extracted and shown in hover.

### SurrealDB DEFINE API (SDB3+)
`DEFINE API /path MIDDLEWARES [...]` — TODO add to grammar/parser/LSP.
