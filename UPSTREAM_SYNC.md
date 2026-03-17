# Upstream Sync

surql-parser extracts its parser source from [SurrealDB](https://github.com/surrealdb/surrealdb). This document explains how the sync works.

## Architecture

```
src/upstream/     AUTO-GENERATED — never edit manually
src/lib.rs        Our public API (stable)
src/compat.rs     Compatibility stubs for engine types
src/config.rs     Parser constants
tests/            Our tests
tools/transform/  AST-level Rust transformer
```

## Running a Sync

```bash
# From local SurrealDB clone (fast):
./scripts/sync-upstream.sh local

# From a specific tag:
./scripts/sync-upstream.sh v3.0.3

# From latest main:
./scripts/sync-upstream.sh main
```

The script:
1. Copies `syn/`, `sql/`, `fmt/` from SurrealDB source
2. Copies `language-tests/tests/parsing/` as test fixtures
3. Runs the AST transformer (`tools/transform/`) with `mappings.toml` rules
4. Applies post-transform patches (sed + file overrides)
5. Checks compilation

## When a Sync Breaks

If SurrealDB adds a new `crate::*` module reference:
1. CI will fail with "unresolved import `crate::something_new`"
2. Add a mapping to `tools/transform/mappings.toml`
3. Or add a stub to `src/compat.rs`
4. Re-run sync

If SurrealDB adds a new type that compat doesn't have:
1. CI will fail with "cannot find type `NewType`"
2. Add the type to `src/compat.rs` (usually 1-5 lines)
3. Re-run sync

## Key Files

- `tools/transform/mappings.toml` — import rewrite rules, strip patterns
- `tools/transform/src/main.rs` — AST transformer (syn + prettyplease)
- `scripts/sync-upstream.sh` — orchestrator script
- `src/compat.rs` — stubs for engine-internal types
- `transforms/patches/module.rs.override` — static file overrides
- `DIRTY_HACKS.md` — tracks temporary workarounds
- `UPSTREAM_HASH` — hash of last synced source (skip if unchanged)
