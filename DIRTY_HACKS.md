# Dirty Hacks Tracker

> Temporary workarounds that MUST be fixed before v1.0 publish.
> Each entry has a condition for removal.

## Active

### 1. sed post-transform patches in sync-upstream.sh
- **What**: Text-level sed after AST transformer for edge cases (multi-imports, `ExprIdioms`, deref `*MAX_DEPTH`)
- **Why**: AST transformer can't handle `use crate::{a, b}` grouped imports or bare module refs
- **Fix**: Improve transformer to handle UseTree properly, or use two-pass approach
- **Remove when**: Transformer handles all import styles

### 2. module.rs.override (static file override)
- **What**: `transforms/patches/module.rs.override` replaces auto-generated module.rs entirely
- **Why**: Too many interleaved expr/catalog From impls that break with brace-matching strip
- **Fix**: Improve python stripping to properly track brace depth across From impls
- **Remove when**: Transformer or python script correctly strips From impls without orphan braces

### 3. Python-based expr:: stripping in sync script
- **What**: Python script removes lines/blocks containing `expr::` from ast.rs, literal.rs
- **Why**: Text-level import strip removed `use crate::{..., expr}` but left From impls using bare `expr::X`
- **Fix**: AST transformer should track stripped modules and remove any impl blocks that become unresolvable
- **Remove when**: Transformer has proper "dead import" analysis

### 4. val wrapper types with pub fields (compat::val)
- **What**: Our own Duration, Datetime, Uuid etc. wrappers because surrealdb-types 3.0.4 has private tuple fields
- **Why**: Parser source constructs `Duration(inner)` directly but published crate doesn't allow it
- **Fix**: Switch to `surrealdb-types = "3.1"` when released (constructors should be pub)
- **Remove when**: surrealdb-types 3.1+ published on crates.io with pub constructors

### 5. fmt_non_finite_f64 compat stub
- **What**: Manual impl of `fmt_non_finite_f64` in compat::fmt
- **Why**: Function exists in surrealdb-types 3.1.0-alpha but not in published 3.0.4
- **Fix**: Remove compat stub, use `surrealdb_types::fmt_non_finite_f64` directly
- **Remove when**: surrealdb-types 3.1+ published

## Resolution Attempt (2026-03-18)

Tried to consolidate all sed/python patches into the Rust transformer.
The text-level block stripper becomes too aggressive when handling
both `expr::` and `crate::expr::` patterns — breaks file parsing for
define/database.rs, define/field.rs, etc. (7 files fail to parse).

The root cause: the block stripper uses line-level heuristics (brace counting)
which fails on complex interleaved impl blocks. A proper fix needs a
two-pass approach: first text-replace all imports, then AST-parse and
remove orphaned impls at the AST level (which understands brace nesting).

For now, the sed/python patches in sync-upstream.sh are the working solution.
They're ugly but reliable and well-documented.

## Resolved

(none yet)
