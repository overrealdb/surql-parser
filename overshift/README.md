<h1 align="center">overshift</h1>

<p align="center">
  Shared migration engine for the <a href="https://github.com/overrealdb">overrealdb</a> ecosystem.
</p>

<p align="center">
  <a href="https://github.com/overrealdb/surql-parser/actions/workflows/ci.yml"><img src="https://github.com/overrealdb/surql-parser/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://codecov.io/gh/overrealdb/surql-parser"><img src="https://codecov.io/gh/overrealdb/surql-parser/graph/badge.svg" alt="codecov" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License" /></a>
</p>

Manages **declarative schema** (`DEFINE ... OVERWRITE`, re-applied at startup) and **imperative migrations** (versioned, checksummed, one-shot) for SurrealDB 3+.

## Features

- **Manifest-driven** — `manifest.toml` defines namespace, database, and schema modules with dependency ordering
- **Distributed lock** — Shedlock-style leader election via `leader_lock` table (60s expiry, scope-parameterized)
- **Checksum validation** — SHA-256 integrity checks prevent modified migrations from being re-applied
- **Dry-run / plan mode** — preview what will be done before applying
- **Schema snapshot** — `generated/current.surql` for CI verification
- **Changelog** — audit trail of all applied migrations and schema modules in `_system` DB
- **Function validation** — verify all `fn::*` exist in database after schema apply

## Project structure

```
surql/
├── manifest.toml               # namespace, database, module config
├── schema/                     # DECLARATIVE (re-applied with OVERWRITE)
│   ├── _shared/
│   │   └── analyzers.surql
│   └── entity/
│       ├── table.surql
│       ├── indexes.surql
│       └── fn.surql
├── migrations/                 # IMPERATIVE (one-shot, versioned)
│   ├── v001_initial_seed.surql
│   └── v002_backfill.surql
└── generated/
    └── current.surql           # auto-generated schema snapshot
```

## manifest.toml

```toml
[meta]
ns = "myapp"
db = "main"
system_db = "_system"
surrealdb = ">=3.0.0"

[[modules]]
name = "_shared"
path = "schema/_shared"

[[modules]]
name = "entity"
path = "schema/entity"
depends_on = ["_shared"]
```

## Library usage

```rust
use surrealdb::engine::any;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = any::connect("ws://localhost:8000").await?;
    db.signin(surrealdb::opt::auth::Root {
        username: "root".into(),
        password: "root".into(),
    }).await?;

    let manifest = overshift::Manifest::load("surql/")?;

    // Dry-run: preview what will be done
    let plan = overshift::plan(&db, &manifest).await?;
    plan.print();

    // Apply: migrations + schema + validation
    let result = plan.apply(&db).await?;
    println!(
        "Applied {} migrations, {} schema modules",
        result.applied_migrations, result.applied_modules,
    );

    Ok(())
}
```

## CLI

```sh
# Install
cargo install overshift --features cli

# Preview changes (dry-run)
overshift plan surql/

# Apply migrations + schema
overshift apply surql/

# Generate schema snapshot
overshift snapshot surql/

# Check snapshot is up to date (CI)
overshift snapshot surql/ --check

# Validate functions exist in database
overshift validate surql/
```

## Startup sequence

1. Connect to SurrealDB
2. `USE NS {ns} DB {system_db}`
3. Bootstrap `_system` tables (`migration_lock`, `leader_lock`, `shedlock`, `changelog`)
4. Acquire distributed lock
5. Run pending imperative migrations (checksummed)
6. `USE NS {ns} DB {db}`
7. Apply declarative schema modules (in dependency order)
8. Validate all `fn::*` exist
9. Release lock

## Testing

```sh
# Unit tests (instant, no DB)
cargo test

# Integration tests with Docker (testcontainers)
cargo test --features validate-docker

# Full CI
cargo make ci
```

## License

Apache-2.0
