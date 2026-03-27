-- Migration 001: Bootstrap overrealdb platform
-- Creates the initial agent and project tables.

CREATE agent:overseer SET
	name = 'overseer',
	role = 'deployer',
	model = 'claude-opus-4-6',
	active = true,
	last_seen = time::now();

CREATE agent:reviewer SET
	name = 'reviewer',
	role = 'reviewer',
	model = 'claude-sonnet-4-6',
	active = true;

CREATE project:surql_parser SET
	name = 'surql-parser',
	description = 'Standalone SurrealQL parser for the overrealdb ecosystem',
	owner = agent:overseer,
	ns = 'overrealdb',
	db = 'main',
	endpoint = 'ws://localhost:8000';
