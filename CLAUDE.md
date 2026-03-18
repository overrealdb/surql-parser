# CLAUDE.md

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
