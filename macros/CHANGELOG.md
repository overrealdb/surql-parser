# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/overrealdb/surql-parser/compare/surql-macros-v0.1.1...surql-macros-v0.1.2) - 2026-03-28

### Added

- *(macros)* add mode = "query" to #[surql_function] for auto-generated bodies

## [0.1.1](https://github.com/overrealdb/surql-parser/compare/surql-macros-v0.1.0...surql-macros-v0.1.1) - 2026-03-27

### Added

- *(macros)* add query inference and schema-aware type checking

### Fixed

- *(macros)* use workspace target dir for trybuild fixtures in CI

### Other

- Fix audit issues: crash tests, swallowed errors, formatting range
- Add surql_query! macro with compile-time parameter validation
