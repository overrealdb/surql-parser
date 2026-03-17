#!/bin/bash
set -euo pipefail

# surql-parser upstream sync script
# Usage: ./scripts/sync-upstream.sh [surrealdb-ref]
#   surrealdb-ref: git tag or branch (default: main)
#
# What it does:
#   1. Clones SurrealDB at the specified ref (shallow)
#   2. Checks if syn/ + sql/ actually changed (hash comparison)
#   3. Copies parser source into src/upstream/
#   4. Runs AST transformer (tools/transform/)
#   5. Runs cargo check to verify compilation

SURREALDB_REF="${1:-main}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
TEMP=$(mktemp -d)
trap "rm -rf $TEMP" EXIT

echo "=== surql-parser sync from SurrealDB $SURREALDB_REF ==="
echo ""

# If we have a local SurrealDB clone, use it (faster)
# Set SURREALDB_LOCAL env var to your local clone path
LOCAL_SURREALDB="${SURREALDB_LOCAL:-}"
if [ -n "$LOCAL_SURREALDB" ] && [ -d "$LOCAL_SURREALDB/.git" ]; then
	echo "[1/6] Using local SurrealDB clone at $LOCAL_SURREALDB"
	SRC="$LOCAL_SURREALDB/surrealdb/core/src"
	if [ "$SURREALDB_REF" != "local" ]; then
		echo "  (To use a specific ref, clone fresh: set LOCAL_SURREALDB='' )"
	fi
else
	echo "[1/6] Cloning SurrealDB $SURREALDB_REF (shallow)..."
	git clone --depth 1 --branch "$SURREALDB_REF" \
		"https://github.com/surrealdb/surrealdb.git" \
		"$TEMP/surrealdb" 2>&1 | tail -1
	SRC="$TEMP/surrealdb/surrealdb/core/src"
fi

# Verify source exists
if [ ! -d "$SRC/syn" ] || [ ! -d "$SRC/sql" ]; then
	echo "ERROR: Cannot find $SRC/syn/ and $SRC/sql/"
	echo "SurrealDB source structure may have changed."
	exit 1
fi

# 2. Hash check — skip if nothing changed
echo "[2/6] Checking for changes in parser source..."
NEW_HASH=$(find "$SRC/syn" "$SRC/sql" "$SRC/fmt" -name "*.rs" -exec shasum -a 256 {} + \
	| sort | shasum -a 256 | cut -d' ' -f1)
OLD_HASH=$(cat "$ROOT/UPSTREAM_HASH" 2>/dev/null || echo "none")

if [ "$NEW_HASH" = "$OLD_HASH" ]; then
	echo "  No changes in syn/ or sql/ modules. Nothing to do."
	exit 0
fi
echo "  Changes detected (hash: ${NEW_HASH:0:12}... vs ${OLD_HASH:0:12}...)"

# 3. Copy source
echo "[3/6] Copying parser source..."
rm -rf "$ROOT/src/upstream"
mkdir -p "$ROOT/src/upstream/syn" "$ROOT/src/upstream/sql"

cp -r "$SRC/syn/"* "$ROOT/src/upstream/syn/"
cp -r "$SRC/sql/"* "$ROOT/src/upstream/sql/"

# Copy language-tests for parsing validation
LANG_TESTS="$(dirname "$SRC")/../../language-tests/tests/parsing"
if [ -d "$LANG_TESTS" ]; then
	rm -rf "$ROOT/tests/fixtures/parsing"
	mkdir -p "$ROOT/tests/fixtures"
	cp -r "$LANG_TESTS" "$ROOT/tests/fixtures/parsing"
	FIXTURE_COUNT=$(find "$ROOT/tests/fixtures/parsing" -name "*.surql" | wc -l | tr -d ' ')
	echo "  Copied $FIXTURE_COUNT parsing test fixtures"
fi

# Also copy supporting modules (type definitions used by parser AST)
# NOTE: catalog/ is NOT copied — too many engine deps. Stubbed in compat.rs instead.
for mod in fmt; do
	if [ -d "$SRC/$mod" ]; then
		mkdir -p "$ROOT/src/upstream/$mod"
		cp -r "$SRC/$mod/"* "$ROOT/src/upstream/$mod/"
	fi
done

# Count what we copied
FILE_COUNT=$(find "$ROOT/src/upstream" -name "*.rs" | wc -l | tr -d ' ')
LINE_COUNT=$(find "$ROOT/src/upstream" -name "*.rs" -exec cat {} + | wc -l | tr -d ' ')
echo "  Copied $FILE_COUNT files ($LINE_COUNT lines)"

# 4. Run AST transformer
echo "[4/6] Running AST transformer..."
TRANSFORM_BIN="$ROOT/tools/transform"
MAPPINGS="$TRANSFORM_BIN/mappings.toml"

if ! cargo build -p surql-transform --quiet 2>/dev/null; then
	echo "  Building transformer..."
	cargo build -p surql-transform 2>&1 | tail -3
fi

cargo run -p surql-transform -- \
	--mappings "$MAPPINGS" \
	--input "$ROOT/src/upstream" \
	2>&1 | while IFS= read -r line; do echo "  $line"; done

# 4b. Post-transform patches (things the AST transformer can't handle)
#
# These are edge cases: multi-imports, bare module refs, version-specific functions.
# If a new pattern appears after SurrealDB update, add it here.
find "$ROOT/src/upstream" -name "*.rs" -exec sed -i '' \
	-e 's/use crate::{catalog, expr};/use crate::compat::catalog;/' \
	-e 's/surrealdb_types::fmt_non_finite_f64/crate::compat::fmt::fmt_non_finite_f64/g' \
	-e 's/use surrealdb_types::{SqlFormat, ToSql, fmt_non_finite_f64}/use surrealdb_types::{SqlFormat, ToSql}; use crate::compat::fmt::fmt_non_finite_f64/g' \
	-e 's/, fmt_non_finite_f64}/}; use crate::compat::fmt::fmt_non_finite_f64/g' \
	-e 's/\*MAX_OBJECT_PARSING_DEPTH/MAX_OBJECT_PARSING_DEPTH/g' \
	-e 's/\*MAX_QUERY_PARSING_DEPTH/MAX_QUERY_PARSING_DEPTH/g' \
	-e '/use.*ExprIdioms/d' \
	-e '/impl From<ExprIdioms>/,/^}/d' \
	-e '/impl From<Idioms> for ExprIdioms/,/^}/d' \
	-e 's/ExprIdioms/Idioms/g' \
	-e 's/crate::compat::val::Duration(d\.into_inner())/d/g' \
	-e 's/crate::compat::val::Datetime(dt\.into_inner())/dt/g' \
	-e 's/crate::compat::val::Uuid(u\.into_inner())/u/g' \
	-e 's/crate::compat::val::Bytes(b\.into_inner())/b/g' \
	-e 's/crate::compat::val::Regex(r\.into_inner())/r/g' \
	-e 's/use jsonwebtoken/\/\/ use jsonwebtoken/g' \
	-e 's/pub(crate) /pub /g' \
	-e 's/\.into_string()/.to_string()/g' \
	-e 's/Decimal::from_str_normalized/crate::compat::decimal_from_str_normalized/g' \
	{} +

# Remove files that are too coupled to engine execution layer
# fmt/test.rs references crate::expr::LogicalPlan
rm -f "$ROOT/src/upstream/fmt/test.rs"

# Strip remaining bare `expr::` references (From impls that survived).
# These files use `expr::X` (without crate:: prefix) because of multi-import.
# We remove entire impl blocks and functions that reference expr types.

# module.rs: remove all From<expr::*> and From<*> for expr::* impl blocks
if [ -f "$ROOT/src/upstream/sql/module.rs" ]; then
	python3 -c "
import re, sys
src = open(sys.argv[1]).read()
# Remove impl From<expr::*> blocks and impl From<*> for expr::* blocks
src = re.sub(r'impl From<expr::[^{]+\{[^}]*\}', '', src)
src = re.sub(r'impl From<\w+> for expr::[^{]+\{[^}]*\}', '', src)
# Remove use crate::{catalog, expr} if still present (already handled by sed above)
open(sys.argv[1], 'w').write(src)
" "$ROOT/src/upstream/sql/module.rs"
fi

# ast.rs + module.rs + literal.rs: remove all remaining expr:: references
# Using a general-purpose Python script that removes impl blocks referencing expr
for f in "$ROOT/src/upstream/sql/ast.rs" "$ROOT/src/upstream/sql/module.rs" "$ROOT/src/upstream/sql/literal.rs"; do
	[ -f "$f" ] || continue
	python3 -c "
import sys
lines = open(sys.argv[1]).readlines()
result = []
skip_depth = 0
skip = False
for line in lines:
    # Start skipping on impl blocks that reference expr::
    stripped = line.strip()
    if not skip and ('expr::' in line or 'crate::expr' in line):
        if stripped.startswith('impl ') or stripped.startswith('fn ') or stripped.startswith('pub fn '):
            skip = True
            skip_depth = 0
        elif stripped.startswith('use ') or stripped.startswith('//'):
            continue  # remove use/comment lines with expr
        else:
            continue  # remove standalone expr:: lines
    if skip:
        skip_depth += line.count('{') - line.count('}')
        if skip_depth <= 0:
            skip = False
        continue
    result.append(line)
open(sys.argv[1], 'w').write(''.join(result))
" "$f"
done

# literal.rs: remove convert_geometry function that uses crate::expr::Literal
if [ -f "$ROOT/src/upstream/sql/literal.rs" ]; then
	python3 -c "
import re, sys
src = open(sys.argv[1]).read()
# Remove the convert_geometry function and its caller
src = re.sub(r'fn convert_geometry\(map: Vec<ObjectEntry>\)[^}]+(?:\{[^}]*\}[^}]*)*\}', '', src, flags=re.DOTALL)
open(sys.argv[1], 'w').write(src)
" "$ROOT/src/upstream/sql/literal.rs"
fi

# 4b2. Restore pub(crate) for macro re-exports (can't be fully pub)
for f in "$ROOT/src/upstream/syn/error/mac.rs" \
         "$ROOT/src/upstream/syn/parser/mac.rs" \
         "$ROOT/src/upstream/syn/token/mac.rs" \
         "$ROOT/src/upstream/syn/token/keyword.rs"; do
	[ -f "$f" ] && sed -i '' 's/^pub use /pub(crate) use /g' "$f"
done
# Also fix re-exports in mod.rs files that forward macro exports
for f in "$ROOT/src/upstream/syn/error/mod.rs" \
         "$ROOT/src/upstream/syn/parser/mod.rs" \
         "$ROOT/src/upstream/syn/token/mod.rs"; do
	[ -f "$f" ] && sed -i '' -e 's/^pub use mac::/pub(crate) use mac::/g' -e 's/^pub use keyword::/pub(crate) use keyword::/g' "$f"
done

# 4b3. Fix upstream doctest (references internal API without full path)
if [ -f "$ROOT/src/upstream/syn/mod.rs" ]; then
	sed -i '' 's|/// ```$|/// ```ignore|' "$ROOT/src/upstream/syn/mod.rs"
fi

# 4c. Fix type inference for Array::from and Object::from in expression.rs
if [ -f "$ROOT/src/upstream/sql/expression.rs" ]; then
	sed -i '' \
		-e 's/\.map(convert_public_value_to_internal)\.collect(),/.map(convert_public_value_to_internal).collect::<Vec<_>>(),/' \
		-e 's/\.map(|(k, v)| (k, convert_public_value_to_internal(v)))/.map(|(k, v)| (k, convert_public_value_to_internal(v)))/' \
		"$ROOT/src/upstream/sql/expression.rs"
	# Fix Object::from collect — needs BTreeMap type
	python3 -c "
import sys
src = open(sys.argv[1]).read()
src = src.replace(
    '.collect(),\n                ),\n            )\n        }\n        surrealdb_types::RecordIdKey::Range',
    '.collect::<std::collections::BTreeMap<_,_>>(),\n                ),\n            )\n        }\n        surrealdb_types::RecordIdKey::Range'
)
open(sys.argv[1], 'w').write(src)
" "$ROOT/src/upstream/sql/expression.rs"
fi

# 4c2. Fix private constructor calls in expression.rs
# surrealdb-types 3.0.4 has private tuple fields for Array, Object
if [ -f "$ROOT/src/upstream/sql/expression.rs" ]; then
	sed -i '' \
		-e 's/crate::compat::val::Array(/crate::compat::val::Array::from(/g' \
		-e 's/crate::compat::val::Object(/crate::compat::val::Object::from(/g' \
		"$ROOT/src/upstream/sql/expression.rs"
fi

# 4d. Apply file overrides (files too complex to auto-transform)
for override in "$ROOT/transforms/patches/"*.override; do
	[ -f "$override" ] || continue
	basename=$(basename "$override" .override)
	target="$ROOT/src/upstream/sql/$basename"
	if [ -f "$target" ]; then
		cp "$override" "$target"
		echo "  Applied override: $basename"
	fi
done

# 5. Generate upstream/mod.rs
echo "[5/6] Generating module files..."
cat > "$ROOT/src/upstream/mod.rs" << EOF
//! Auto-generated from SurrealDB source. DO NOT EDIT MANUALLY.
//!
//! Synced from: $SURREALDB_REF
//! Hash: $NEW_HASH
//! Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
//!
//! To update, run: ./scripts/sync-upstream.sh [tag]

#![allow(unused_imports, dead_code, unused_variables, unreachable_code)]

pub mod fmt;
pub mod sql;
pub mod syn;
EOF

# Record version and hash
echo "$SURREALDB_REF" > "$ROOT/UPSTREAM_VERSION"
echo "$NEW_HASH" > "$ROOT/UPSTREAM_HASH"

# 6. Try to compile
echo "[6/6] Checking compilation..."
cd "$ROOT"
if cargo check 2>"$TEMP/check_errors.txt"; then
	echo "  ✓ Compilation successful!"
else
	ERROR_COUNT=$(grep -c "^error" "$TEMP/check_errors.txt" 2>/dev/null || echo "?")
	echo "  ✗ Compilation failed ($ERROR_COUNT errors)"
	echo ""
	echo "  First 30 errors:"
	head -60 "$TEMP/check_errors.txt" | grep -A1 "^error" | head -30
	echo ""
	echo "  Fix compilation errors, then run: cargo check"
	echo "  If a new crate:: import appeared, add it to mappings.toml"
fi

echo ""
echo "=== Sync complete ==="
echo "  Version: $SURREALDB_REF"
echo "  Files:   $FILE_COUNT"
echo "  Lines:   $LINE_COUNT"
echo "  Review:  git diff src/upstream/"
