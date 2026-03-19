#!/usr/bin/env python3
"""Extract SurrealQL code blocks from SurrealDB docs MDX files.

Clones the docs repo (shallow, depth 1) into a temp directory, finds all
.mdx files under src/content/doc-surrealql/, and extracts ```surql and
```sql code blocks as individual .surql fixture files.

Output blocks (Response, Output, API DEFINITION, etc.) are skipped.

Usage:
    python3 scripts/extract-doc-examples.py [--docs-dir /path/to/existing/clone]
"""

import os
import re
import sys
import shutil
import tempfile
import subprocess
from pathlib import Path

DOCS_REPO = "https://github.com/surrealdb/docs.surrealdb.com"
DOCS_SUBDIR = "src/content/doc-surrealql"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "tests" / "fixtures" / "doc-examples"

SKIP_TITLES = {
    "response",
    "output",
    "api definition",
    "sample output",
    "possible output",
    "error response",
    "result",
    "surrealql syntax",
}

CODE_BLOCK_RE = re.compile(
    r"^```(?:surql|sql)([^\n]*)\n(.*?)^```",
    re.MULTILINE | re.DOTALL,
)


def should_skip_block(attributes: str) -> bool:
    """Return True if the code block is an output/response block."""
    title_match = re.search(r'title\s*=\s*"([^"]*)"', attributes)
    if title_match:
        title = title_match.group(1).lower().strip()
        if any(skip in title for skip in SKIP_TITLES):
            return True
    output_match = re.search(r'output\s*=\s*"([^"]*)"', attributes)
    if output_match:
        return True
    return False


def extract_blocks_from_file(mdx_path: Path) -> list[str]:
    """Extract all surql/sql code blocks from an MDX file."""
    content = mdx_path.read_text(encoding="utf-8", errors="replace")
    blocks = []
    for match in CODE_BLOCK_RE.finditer(content):
        attributes = match.group(1)
        code = match.group(2).strip()
        if should_skip_block(attributes):
            continue
        if not code:
            continue
        blocks.append(code)
    return blocks


def sanitize_filename(name: str) -> str:
    """Convert an MDX filename stem into a safe fixture name."""
    return re.sub(r"[^a-zA-Z0-9_-]", "_", name)


def main():
    docs_dir = None
    cleanup_dir = None

    if "--docs-dir" in sys.argv:
        idx = sys.argv.index("--docs-dir")
        if idx + 1 < len(sys.argv):
            docs_dir = Path(sys.argv[idx + 1])

    if docs_dir is None:
        cleanup_dir = tempfile.mkdtemp(prefix="surql-docs-")
        docs_dir = Path(cleanup_dir) / "docs"
        print(f"Cloning {DOCS_REPO} into {docs_dir} ...")
        subprocess.run(
            ["git", "clone", "--depth", "1", DOCS_REPO, str(docs_dir)],
            check=True,
            capture_output=True,
        )

    try:
        surql_dir = docs_dir / DOCS_SUBDIR
        if not surql_dir.is_dir():
            print(f"ERROR: {surql_dir} not found", file=sys.stderr)
            sys.exit(1)

        mdx_files = sorted(surql_dir.rglob("*.mdx"))
        print(f"Found {len(mdx_files)} MDX files")

        if OUTPUT_DIR.exists():
            for f in OUTPUT_DIR.glob("*.surql"):
                f.unlink()
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

        total_blocks = 0
        for mdx_path in mdx_files:
            rel = mdx_path.relative_to(surql_dir)
            parts = list(rel.parent.parts) + [rel.stem]
            base_name = sanitize_filename("_".join(parts))

            blocks = extract_blocks_from_file(mdx_path)
            for i, block in enumerate(blocks):
                fixture_name = f"{base_name}_{i}.surql"
                fixture_path = OUTPUT_DIR / fixture_name
                fixture_path.write_text(block + "\n", encoding="utf-8")
                total_blocks += 1

        print(f"Extracted {total_blocks} code blocks to {OUTPUT_DIR}")

    finally:
        if cleanup_dir:
            shutil.rmtree(cleanup_dir, ignore_errors=True)


if __name__ == "__main__":
    main()
