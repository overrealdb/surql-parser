# CLAUDE.md

# Code Quality Rules (MANDATORY)

## Naming
- NEVER use: helper, utils, handler, manager, misc, data, info, process
- Every function/class name must describe WHAT it does, not its role
- If you can't name it specifically, the abstraction is wrong

## Error Handling
- NEVER swallow errors with empty catch/except blocks
- NEVER add silent default fallback values (?? 'default', || [])
- If a value can be undefined, either: throw, handle explicitly, or 
  document WHY a default is safe here
- Every catch block must: handle meaningfully, log+rethrow, or have 
  a comment explaining why swallowing is intentional

## Testing
- Write failing tests BEFORE implementation (TDD)
- Tests must verify BEHAVIOR through public interfaces, not implementation
- Every test name describes the scenario: "should_reject_expired_tokens"
- No tests that could pass if the feature is broken
- Test edge cases: null, empty, negative, concurrent, boundary values

## Before Stopping
- Run the test suite. Do not claim "done" with failing tests.
- Review your own changes for the patterns listed above
- Mention any shortcuts, technical debt, or known issues proactively

## Compact Instructions
When compacting, preserve: all file paths modified, architectural decisions 
with rationale, any error messages encountered, and the current task status.

## Testing
- ALWAYS write tests for new public functions
- ALWAYS run `cargo test` before committing
- Use testcontainers for SurrealDB tests, NOT in-memory
- Add proptest for any parsing/serialization code
- Never edit `src/upstream/` — auto-generated from SurrealDB

## Code Style
- No inline SurrealQL in Rust code — use fn::* or include_str!
- No unnecessary comments (code should be self-documenting)
- No over-engineering — solve the current problem, not hypothetical future ones
- hard_tabs = true, max_width = 100

## Quality
- `cargo make ci` must pass before creating PR
- Zero clippy warnings
- Zero fmt diffs
- Never push directly to main — always create a PR

## Architecture
- `src/builtins_generated.rs` — auto-generated, regenerate with `cargo make sync-builtins`
- `src/upstream/` — auto-generated from SurrealDB, regenerate with `cargo make sync-main`
- `tree-sitter-surrealql/` — our grammar (Apache-2.0), generate with `npx tree-sitter generate`
- LSP binary: `cargo install --path lsp --force` — Zed uses installed binary, NOT dev build

## LSP Development
- After changing LSP code, MUST run `cargo install --path lsp --force` for Zed to pick up changes
- `cargo fmt --check` must run from repo root (fails inside tree-sitter-surrealql/)
- Schema merge: SCHEMAFULL wins, fields/indexes deduplicated by name
- Clippy must pass for ALL workspace crates: surql-parser, surql-lsp, surql-transform
