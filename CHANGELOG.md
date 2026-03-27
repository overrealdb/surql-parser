# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/overrealdb/surql-parser/compare/surql-parser-v0.1.2...surql-parser-v0.1.3) - 2026-03-27

### Added

- *(core)* add schema graph, linting, diff, params and lookup

### Fixed

- exclude surql-lsp and surql-mcp from release-plz
- *(ci)* fix gitleaks allowlist key, add Unlicense to deny allow list
- *(ci)* use exceptions for unlicensed surrealdb crates, fix gitleaks config
- *(ci)* allow unlicensed surrealdb crates in deny, allowlist doc example keys

### Other

- update workspace config, CI, examples, and add project icons
- Integrate overshift into MCP and manifest-aware LSP scoping
- Add NS/DB scope filtering to SchemaGraph for smart context
- Fix duplicate fields in schema merge, dedup in from_definitions
- Move overshift migration engine into workspace
- Auto-detect overshift manifest.toml in LSP workspace
- Add overshift manifest tool to MCP, cross-file record link diagnostics
- Add NS/DB scoping to SchemaGraph, track USE statements in definitions
- Add /surql-relations slash command, fix hover doc URLs, document SurrealDB 3.x target
- Add MCP server, Zed extension with /surql-schema, fix memory leak and formatter
- Add DualEngine embedded SurrealDB, schema-aware diagnostics, and LSP enhancements
- Fix Zed extension: remove broken outline.scm and indents.scm
- Add Wasm extension for Zed LSP integration
- Remove Zed build artifacts, add to gitignore
- Fix Zed extension: indents.scm uses @indent/@outdent not @indent.start
- Refactor schema_graph: Info→Def, Vec→Iterator, PathBuf→Arc<Path>
- Fix 3 critical issues from self-review
- Add Zed extension for SurrealQL
- Add 9 integration tests with full JSON-RPC protocol
- Fix remaining cut corners: keywords from enum, lexer source locations, assertion tests
- Harden LSP quality: error recovery, token-based completions, keyword sync
- Add field completions, signature help, go-to-definition, 130 LSP tests
- Add SchemaGraph, parse_for_diagnostics, and LSP server
- Add surql_query! macro with compile-time parameter validation
- Update all dependencies from Dependabot PRs
- Fix proptest: backtick-escape generated identifiers
- Upgrade jsonwebtoken 9 → 10 to fix type confusion vulnerability
- Align surql-parser with overtemplate gold standard

## [0.1.2](https://github.com/overrealdb/surql-parser/compare/surql-parser-v0.1.1...surql-parser-v0.1.2) - 2026-03-18

### Other

- Remove unused helper, fail CI on any warning

## [0.1.1](https://github.com/overrealdb/surql-parser/compare/surql-parser-v0.1.0...surql-parser-v0.1.1) - 2026-03-18

### Other

- Replace manual release with release-plz + separate CLI builds
- Add compile-time SurrealQL tools: surql-macros + build helpers
