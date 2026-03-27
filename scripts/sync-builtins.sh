#!/bin/bash
set -euo pipefail

# Extract built-in function documentation from SurrealDB docs.
# Usage: ./scripts/sync-builtins.sh [docs-ref]
#
# Clones docs.surrealdb.com at the given ref, parses MDX files,
# and generates src/builtins_generated.rs.

DOCS_REF="${1:-main}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
TEMP=$(mktemp -d)
trap "rm -rf $TEMP" EXIT

echo "=== Sync built-in function docs from SurrealDB docs ($DOCS_REF) ==="
echo ""

# 1. Clone docs repo (shallow)
echo "[1/3] Cloning docs.surrealdb.com ($DOCS_REF)..."
git clone --depth 1 --branch "$DOCS_REF" \
	"https://github.com/surrealdb/docs.surrealdb.com.git" \
	"$TEMP/docs" 2>&1 | tail -1

DOCS_DIR="$TEMP/docs/src/content/doc-surrealql/functions/database"
if [ ! -d "$DOCS_DIR" ]; then
	echo "ERROR: Functions directory not found at $DOCS_DIR"
	echo "SurrealDB docs structure may have changed."
	exit 1
fi

MDX_COUNT=$(find "$DOCS_DIR" -name "*.mdx" | wc -l | tr -d ' ')
echo "  Found $MDX_COUNT MDX files"

# 2. Parse MDX files → Rust
echo "[2/3] Parsing function definitions..."
python3 "$SCRIPT_DIR/parse-builtins.py" "$DOCS_DIR" "$ROOT/src/builtins_generated.rs"

# 3. Verify compilation
echo "[3/3] Checking compilation..."
cd "$ROOT"
if cargo check 2>/dev/null; then
	echo "  ✓ Compilation successful!"
else
	echo "  ✗ Compilation failed — check src/builtins_generated.rs"
	exit 1
fi

FN_COUNT=$(grep -c "BuiltinFn {" "$ROOT/src/builtins_generated.rs" || echo "0")
echo ""
echo "=== Sync complete ==="
echo "  Functions: $FN_COUNT"
echo "  Output:    src/builtins_generated.rs"
