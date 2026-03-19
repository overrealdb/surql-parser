-- Migration 001: Initial schema
-- Creates the core tables for the application.

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD age ON user TYPE int DEFAULT 0;
DEFINE FIELD active ON user TYPE bool DEFAULT true;
DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX user_email ON user FIELDS email UNIQUE;

DEFINE TABLE post SCHEMAFULL;
DEFINE FIELD title ON post TYPE string;
DEFINE FIELD body ON post TYPE string;
DEFINE FIELD author ON post TYPE record<user>;
DEFINE FIELD created_at ON post TYPE datetime DEFAULT time::now();

DEFINE FUNCTION fn::greet($name: string) -> string {
	RETURN 'Hello, ' + $name + '!';
};
