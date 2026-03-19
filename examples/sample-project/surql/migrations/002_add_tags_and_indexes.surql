-- Migration 002: Add tags, categories, and indexes
-- Extends the schema with tagging and search support.

DEFINE FIELD tags ON post TYPE array<string> DEFAULT [];
DEFINE FIELD published ON post TYPE bool DEFAULT false;
DEFINE FIELD slug ON post TYPE option<string>;
DEFINE INDEX post_slug ON post FIELDS slug UNIQUE;

DEFINE TABLE category SCHEMAFULL;
DEFINE FIELD name ON category TYPE string;
DEFINE FIELD description ON category TYPE option<string>;
DEFINE FIELD parent ON category TYPE option<record<category>>;
DEFINE INDEX cat_name ON category FIELDS name UNIQUE;

DEFINE FIELD category ON post TYPE option<record<category>>;

DEFINE FIELD bio ON user TYPE option<string>;
DEFINE FIELD avatar ON user TYPE option<string>;
DEFINE INDEX user_name ON user FIELDS name;
