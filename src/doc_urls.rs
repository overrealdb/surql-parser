//! Auto-generated documentation URL mapping for SurrealQL.
//! Generated from docs.surrealdb.com content on 2026-03-23.
//!
//! Re-generate by running the doc_urls extraction task against
//! the docs repo at `docs.surrealdb.com/src/content/doc-surrealql/`.

/// Base URL for SurrealQL documentation. Intentionally NOT used in match arms below:
/// each URL is spelled out in full for grep-ability (searching for a specific URL
/// should find the exact match arm).
#[allow(dead_code)]
const BASE: &str = "https://surrealdb.com/docs/surrealql";

/// Get the documentation URL for a SurrealQL keyword, statement, function, or clause.
///
/// Accepts keywords in any case. Compound keywords can be space-separated
/// (e.g., `"define table"`, `"order by"`, `"live select"`).
///
/// For function namespaces, pass the namespace with a trailing `::` or without
/// (e.g., `"string"` or `"string::"`). For specific function lookups, use
/// [`doc_url_for_function`] instead.
pub fn doc_url(keyword: &str) -> Option<&'static str> {
	let lower = keyword.to_lowercase();
	let trimmed = lower.trim();
	match trimmed {
		// ── Statements ──────────────────────────────────────────────
		"access" => Some("https://surrealdb.com/docs/surrealql/statements/access"),
		"begin" | "begin transaction" => {
			Some("https://surrealdb.com/docs/surrealql/statements/begin")
		}
		"break" => Some("https://surrealdb.com/docs/surrealql/statements/break"),
		"cancel" | "cancel transaction" => {
			Some("https://surrealdb.com/docs/surrealql/statements/cancel")
		}
		"commit" | "commit transaction" => {
			Some("https://surrealdb.com/docs/surrealql/statements/commit")
		}
		"continue" => Some("https://surrealdb.com/docs/surrealql/statements/continue"),
		"create" => Some("https://surrealdb.com/docs/surrealql/statements/create"),
		"delete" => Some("https://surrealdb.com/docs/surrealql/statements/delete"),
		"explain" => Some("https://surrealdb.com/docs/surrealql/statements/explain"),
		"for" => Some("https://surrealdb.com/docs/surrealql/statements/for"),
		"if" | "if else" | "else" => Some("https://surrealdb.com/docs/surrealql/statements/ifelse"),
		"info" => Some("https://surrealdb.com/docs/surrealql/statements/info"),
		"insert" => Some("https://surrealdb.com/docs/surrealql/statements/insert"),
		"kill" => Some("https://surrealdb.com/docs/surrealql/statements/kill"),
		"let" => Some("https://surrealdb.com/docs/surrealql/statements/let"),
		"live" | "live select" => Some("https://surrealdb.com/docs/surrealql/statements/live"),
		"rebuild" => Some("https://surrealdb.com/docs/surrealql/statements/rebuild"),
		"relate" => Some("https://surrealdb.com/docs/surrealql/statements/relate"),
		"remove" => Some("https://surrealdb.com/docs/surrealql/statements/remove"),
		"return" => Some("https://surrealdb.com/docs/surrealql/statements/return"),
		"select" => Some("https://surrealdb.com/docs/surrealql/statements/select"),
		"show" | "show changes" => Some("https://surrealdb.com/docs/surrealql/statements/show"),
		"sleep" => Some("https://surrealdb.com/docs/surrealql/statements/sleep"),
		"throw" => Some("https://surrealdb.com/docs/surrealql/statements/throw"),
		"update" => Some("https://surrealdb.com/docs/surrealql/statements/update"),
		"upsert" => Some("https://surrealdb.com/docs/surrealql/statements/upsert"),
		"use" => Some("https://surrealdb.com/docs/surrealql/statements/use"),

		// ── DEFINE variants ─────────────────────────────────────────
		"define" => Some("https://surrealdb.com/docs/surrealql/statements/define"),
		"define access" => Some("https://surrealdb.com/docs/surrealql/statements/define/access"),
		"define access bearer" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/access/bearer")
		}
		"define access jwt" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/access/jwt")
		}
		"define access record" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/access/record")
		}
		"define analyzer" | "analyzer" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/analyzer")
		}
		"define api" => Some("https://surrealdb.com/docs/surrealql/statements/define/api"),
		"define bucket" | "bucket" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/bucket")
		}
		"define config" | "config" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/config")
		}
		"define database" | "define db" | "database" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/database")
		}
		"define event" | "event" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/event")
		}
		"define field" | "field" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/field")
		}
		"define function" | "function" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/function")
		}
		"define index" | "index" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/indexes")
		}
		"define module" | "module" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/module")
		}
		"define namespace" | "define ns" | "namespace" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/namespace")
		}
		"define param" => Some("https://surrealdb.com/docs/surrealql/statements/define/param"),
		"define sequence" | "sequence" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/sequence")
		}
		"define table" | "table" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/table")
		}
		"define user" | "user" => {
			Some("https://surrealdb.com/docs/surrealql/statements/define/user")
		}

		// ── ALTER variants ──────────────────────────────────────────
		"alter" => Some("https://surrealdb.com/docs/surrealql/statements/alter"),
		"alter database" | "alter db" => {
			Some("https://surrealdb.com/docs/surrealql/statements/alter/database")
		}
		"alter field" => Some("https://surrealdb.com/docs/surrealql/statements/alter/field"),
		"alter index" => Some("https://surrealdb.com/docs/surrealql/statements/alter/indexes"),
		"alter namespace" | "alter ns" => {
			Some("https://surrealdb.com/docs/surrealql/statements/alter/namespace")
		}
		"alter sequence" => Some("https://surrealdb.com/docs/surrealql/statements/alter/sequence"),
		"alter system" => Some("https://surrealdb.com/docs/surrealql/statements/alter/system"),
		"alter table" => Some("https://surrealdb.com/docs/surrealql/statements/alter/table"),

		// ── Clauses ─────────────────────────────────────────────────
		"explain clause" => Some("https://surrealdb.com/docs/surrealql/clauses/explain"),
		"fetch" => Some("https://surrealdb.com/docs/surrealql/clauses/fetch"),
		"from" => Some("https://surrealdb.com/docs/surrealql/clauses/from"),
		"group by" | "group" => Some("https://surrealdb.com/docs/surrealql/clauses/group-by"),
		"limit" => Some("https://surrealdb.com/docs/surrealql/clauses/limit"),
		"omit" => Some("https://surrealdb.com/docs/surrealql/clauses/omit"),
		"order by" | "order" => Some("https://surrealdb.com/docs/surrealql/clauses/order-by"),
		"split" => Some("https://surrealdb.com/docs/surrealql/clauses/split"),
		"where" => Some("https://surrealdb.com/docs/surrealql/clauses/where"),
		"with" => Some("https://surrealdb.com/docs/surrealql/clauses/with"),

		// ── Data types / datamodel ──────────────────────────────────
		"array" | "arrays" => Some("https://surrealdb.com/docs/surrealql/datamodel/arrays"),
		"bool" | "boolean" | "booleans" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/booleans")
		}
		"bytes" => Some("https://surrealdb.com/docs/surrealql/datamodel/bytes"),
		"casting" | "cast" => Some("https://surrealdb.com/docs/surrealql/datamodel/casting"),
		"closure" | "closures" => Some("https://surrealdb.com/docs/surrealql/datamodel/closures"),
		"datetime" | "datetimes" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/datetimes")
		}
		"duration" | "durations" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/durations")
		}
		"file" | "files" => Some("https://surrealdb.com/docs/surrealql/datamodel/files"),
		"formatters" | "formatter" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/formatters")
		}
		"future" | "futures" => Some("https://surrealdb.com/docs/surrealql/datamodel/futures"),
		"geometry" | "geometries" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/geometries")
		}
		"idiom" | "idioms" => Some("https://surrealdb.com/docs/surrealql/datamodel/idioms"),
		"record id" | "record ids" | "ids" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/ids")
		}
		"literal" | "literals" => Some("https://surrealdb.com/docs/surrealql/datamodel/literals"),
		"none" | "null" | "none and null" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/none-and-null")
		}
		"number" | "numbers" | "int" | "float" | "decimal" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/numbers")
		}
		"object" | "objects" => Some("https://surrealdb.com/docs/surrealql/datamodel/objects"),
		"range" | "ranges" => Some("https://surrealdb.com/docs/surrealql/datamodel/ranges"),
		"record" | "records" | "record link" | "record links" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/records")
		}
		"reference" | "references" | "record reference" | "record references" => {
			Some("https://surrealdb.com/docs/surrealql/datamodel/references")
		}
		"regex" => Some("https://surrealdb.com/docs/surrealql/datamodel/regex"),
		"set" | "sets" => Some("https://surrealdb.com/docs/surrealql/datamodel/sets"),
		"string" | "strings" => Some("https://surrealdb.com/docs/surrealql/datamodel/strings"),
		"uuid" | "uuids" => Some("https://surrealdb.com/docs/surrealql/datamodel/uuid"),
		"value" | "values" => Some("https://surrealdb.com/docs/surrealql/datamodel/values"),
		"data types" | "datamodel" => Some("https://surrealdb.com/docs/surrealql/datamodel"),

		// ── Function namespaces ─────────────────────────────────────
		"api::" | "api functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/api")
		}
		"array::" | "array functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/array")
		}
		"bytes::" | "bytes functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/bytes")
		}
		"count" | "count()" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/count")
		}
		"crypto::" | "crypto functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/crypto")
		}
		"duration::" | "duration functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/duration")
		}
		"encoding::" | "encoding functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/encoding")
		}
		"file::" | "file functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/file")
		}
		"geo::" | "geo functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/geo")
		}
		"http::" | "http functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/http")
		}
		"math::" | "math functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/math")
		}
		"meta::" | "meta functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/meta")
		}
		"not()" | "not function" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/not")
		}
		"object::" | "object functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/object")
		}
		"parse::" | "parse functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/parse")
		}
		"rand::" | "rand functions" | "rand" | "rand()" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/rand")
		}
		"record::" | "record functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/record")
		}
		"search::" | "search functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/search")
		}
		"sequence::" | "sequence functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/sequence")
		}
		"session::" | "session functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/session")
		}
		"set::" | "set functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/set")
		}
		"sleep()" | "sleep function" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/sleep")
		}
		"string::" | "string functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/string")
		}
		"time::" | "time functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/time")
		}
		"type::" | "type functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/type")
		}
		"value::" | "value functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/value")
		}
		"vector::" | "vector functions" => {
			Some("https://surrealdb.com/docs/surrealql/functions/database/vector")
		}
		"functions" => Some("https://surrealdb.com/docs/surrealql/functions"),

		// ── ML functions ────────────────────────────────────────────
		"ml" | "ml::" | "machine learning" => {
			Some("https://surrealdb.com/docs/surrealql/functions/ml")
		}
		"ml functions" => Some("https://surrealdb.com/docs/surrealql/functions/ml/functions"),

		// ── Scripting functions ──────────────────────────────────────
		"script" | "scripting" | "embedded scripting" => {
			Some("https://surrealdb.com/docs/surrealql/functions/script")
		}

		// ── Top-level pages ─────────────────────────────────────────
		"operators" | "operator" => Some("https://surrealdb.com/docs/surrealql/operators"),
		"parameters" | "param" | "$param" => {
			Some("https://surrealdb.com/docs/surrealql/parameters")
		}
		"comments" | "comment" => Some("https://surrealdb.com/docs/surrealql/comments"),
		"transactions" | "transaction" => Some("https://surrealdb.com/docs/surrealql/transactions"),
		"demo" | "demo data" => Some("https://surrealdb.com/docs/surrealql/demo"),
		"surrealql" => Some(BASE),

		_ => None,
	}
}

/// Known function namespace prefixes and their documentation URLs.
const FUNCTION_NAMESPACES: &[(&str, &str)] = &[
	(
		"api::",
		"https://surrealdb.com/docs/surrealql/functions/database/api",
	),
	(
		"array::",
		"https://surrealdb.com/docs/surrealql/functions/database/array",
	),
	(
		"bytes::",
		"https://surrealdb.com/docs/surrealql/functions/database/bytes",
	),
	(
		"crypto::",
		"https://surrealdb.com/docs/surrealql/functions/database/crypto",
	),
	(
		"duration::",
		"https://surrealdb.com/docs/surrealql/functions/database/duration",
	),
	(
		"encoding::",
		"https://surrealdb.com/docs/surrealql/functions/database/encoding",
	),
	(
		"file::",
		"https://surrealdb.com/docs/surrealql/functions/database/file",
	),
	(
		"geo::",
		"https://surrealdb.com/docs/surrealql/functions/database/geo",
	),
	(
		"http::",
		"https://surrealdb.com/docs/surrealql/functions/database/http",
	),
	(
		"math::",
		"https://surrealdb.com/docs/surrealql/functions/database/math",
	),
	(
		"meta::",
		"https://surrealdb.com/docs/surrealql/functions/database/meta",
	),
	(
		"object::",
		"https://surrealdb.com/docs/surrealql/functions/database/object",
	),
	(
		"parse::",
		"https://surrealdb.com/docs/surrealql/functions/database/parse",
	),
	(
		"rand::",
		"https://surrealdb.com/docs/surrealql/functions/database/rand",
	),
	(
		"record::",
		"https://surrealdb.com/docs/surrealql/functions/database/record",
	),
	(
		"search::",
		"https://surrealdb.com/docs/surrealql/functions/database/search",
	),
	(
		"sequence::",
		"https://surrealdb.com/docs/surrealql/functions/database/sequence",
	),
	(
		"session::",
		"https://surrealdb.com/docs/surrealql/functions/database/session",
	),
	(
		"set::",
		"https://surrealdb.com/docs/surrealql/functions/database/set",
	),
	(
		"string::",
		"https://surrealdb.com/docs/surrealql/functions/database/string",
	),
	(
		"time::",
		"https://surrealdb.com/docs/surrealql/functions/database/time",
	),
	(
		"type::",
		"https://surrealdb.com/docs/surrealql/functions/database/type",
	),
	(
		"value::",
		"https://surrealdb.com/docs/surrealql/functions/database/value",
	),
	(
		"vector::",
		"https://surrealdb.com/docs/surrealql/functions/database/vector",
	),
];

/// Standalone functions (no namespace prefix) and their documentation URLs.
const STANDALONE_FUNCTIONS: &[(&str, &str)] = &[
	(
		"count",
		"https://surrealdb.com/docs/surrealql/functions/database/count",
	),
	(
		"not",
		"https://surrealdb.com/docs/surrealql/functions/database/not",
	),
	(
		"sleep",
		"https://surrealdb.com/docs/surrealql/functions/database/sleep",
	),
	(
		"rand",
		"https://surrealdb.com/docs/surrealql/functions/database/rand",
	),
];

/// Get the documentation URL for a SurrealQL function by its full qualified name.
///
/// Matches namespaced functions like `"string::len"` or `"array::push"` to
/// their namespace documentation page (e.g., the String functions page).
///
/// Also matches standalone functions like `"count"`, `"not"`, `"sleep"`, `"rand"`.
///
/// # Examples
///
/// ```
/// use surql_parser::doc_urls::doc_url_for_function;
///
/// assert_eq!(
///     doc_url_for_function("string::len"),
///     Some("https://surrealdb.com/docs/surrealql/functions/database/string"),
/// );
/// assert_eq!(
///     doc_url_for_function("array::push"),
///     Some("https://surrealdb.com/docs/surrealql/functions/database/array"),
/// );
/// assert_eq!(
///     doc_url_for_function("count"),
///     Some("https://surrealdb.com/docs/surrealql/functions/database/count"),
/// );
/// assert_eq!(doc_url_for_function("unknown::thing"), None);
/// ```
pub fn doc_url_for_function(name: &str) -> Option<&'static str> {
	let lower = name.to_lowercase();
	let trimmed = lower.trim();

	for &(prefix, url) in FUNCTION_NAMESPACES {
		if trimmed.starts_with(prefix) || trimmed == prefix.trim_end_matches("::") {
			return Some(url);
		}
	}

	for &(standalone, url) in STANDALONE_FUNCTIONS {
		if trimmed == standalone {
			return Some(url);
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_resolve_statement_keywords() {
		assert_eq!(
			doc_url("SELECT"),
			Some("https://surrealdb.com/docs/surrealql/statements/select")
		);
		assert_eq!(
			doc_url("create"),
			Some("https://surrealdb.com/docs/surrealql/statements/create")
		);
		assert_eq!(
			doc_url("UPDATE"),
			Some("https://surrealdb.com/docs/surrealql/statements/update")
		);
		assert_eq!(
			doc_url("DELETE"),
			Some("https://surrealdb.com/docs/surrealql/statements/delete")
		);
		assert_eq!(
			doc_url("INSERT"),
			Some("https://surrealdb.com/docs/surrealql/statements/insert")
		);
		assert_eq!(
			doc_url("UPSERT"),
			Some("https://surrealdb.com/docs/surrealql/statements/upsert")
		);
		assert_eq!(
			doc_url("RELATE"),
			Some("https://surrealdb.com/docs/surrealql/statements/relate")
		);
		assert_eq!(
			doc_url("LET"),
			Some("https://surrealdb.com/docs/surrealql/statements/let")
		);
		assert_eq!(
			doc_url("LIVE SELECT"),
			Some("https://surrealdb.com/docs/surrealql/statements/live")
		);
	}

	#[test]
	fn should_resolve_define_variants() {
		assert_eq!(
			doc_url("DEFINE TABLE"),
			Some("https://surrealdb.com/docs/surrealql/statements/define/table")
		);
		assert_eq!(
			doc_url("table"),
			Some("https://surrealdb.com/docs/surrealql/statements/define/table")
		);
		assert_eq!(
			doc_url("DEFINE FIELD"),
			Some("https://surrealdb.com/docs/surrealql/statements/define/field")
		);
		assert_eq!(
			doc_url("DEFINE INDEX"),
			Some("https://surrealdb.com/docs/surrealql/statements/define/indexes")
		);
		assert_eq!(
			doc_url("DEFINE NAMESPACE"),
			Some("https://surrealdb.com/docs/surrealql/statements/define/namespace")
		);
		assert_eq!(
			doc_url("DEFINE NS"),
			Some("https://surrealdb.com/docs/surrealql/statements/define/namespace")
		);
	}

	#[test]
	fn should_resolve_alter_variants() {
		assert_eq!(
			doc_url("ALTER TABLE"),
			Some("https://surrealdb.com/docs/surrealql/statements/alter/table")
		);
		assert_eq!(
			doc_url("ALTER DATABASE"),
			Some("https://surrealdb.com/docs/surrealql/statements/alter/database")
		);
		assert_eq!(
			doc_url("ALTER DB"),
			Some("https://surrealdb.com/docs/surrealql/statements/alter/database")
		);
		assert_eq!(
			doc_url("ALTER SYSTEM"),
			Some("https://surrealdb.com/docs/surrealql/statements/alter/system")
		);
	}

	#[test]
	fn should_resolve_clauses() {
		assert_eq!(
			doc_url("WHERE"),
			Some("https://surrealdb.com/docs/surrealql/clauses/where")
		);
		assert_eq!(
			doc_url("ORDER BY"),
			Some("https://surrealdb.com/docs/surrealql/clauses/order-by")
		);
		assert_eq!(
			doc_url("GROUP BY"),
			Some("https://surrealdb.com/docs/surrealql/clauses/group-by")
		);
		assert_eq!(
			doc_url("LIMIT"),
			Some("https://surrealdb.com/docs/surrealql/clauses/limit")
		);
		assert_eq!(
			doc_url("FETCH"),
			Some("https://surrealdb.com/docs/surrealql/clauses/fetch")
		);
		assert_eq!(
			doc_url("SPLIT"),
			Some("https://surrealdb.com/docs/surrealql/clauses/split")
		);
	}

	#[test]
	fn should_resolve_data_types() {
		assert_eq!(
			doc_url("string"),
			Some("https://surrealdb.com/docs/surrealql/datamodel/strings")
		);
		assert_eq!(
			doc_url("number"),
			Some("https://surrealdb.com/docs/surrealql/datamodel/numbers")
		);
		assert_eq!(
			doc_url("datetime"),
			Some("https://surrealdb.com/docs/surrealql/datamodel/datetimes")
		);
		assert_eq!(
			doc_url("uuid"),
			Some("https://surrealdb.com/docs/surrealql/datamodel/uuid")
		);
		assert_eq!(
			doc_url("geometry"),
			Some("https://surrealdb.com/docs/surrealql/datamodel/geometries")
		);
	}

	#[test]
	fn should_resolve_function_namespaces_via_doc_url() {
		assert_eq!(
			doc_url("string::"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/string")
		);
		assert_eq!(
			doc_url("array::"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/array")
		);
		assert_eq!(
			doc_url("math::"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/math")
		);
		assert_eq!(
			doc_url("count"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/count")
		);
	}

	#[test]
	fn should_resolve_specific_functions() {
		assert_eq!(
			doc_url_for_function("string::len"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/string")
		);
		assert_eq!(
			doc_url_for_function("array::push"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/array")
		);
		assert_eq!(
			doc_url_for_function("math::sum"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/math")
		);
		assert_eq!(
			doc_url_for_function("crypto::md5"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/crypto")
		);
		assert_eq!(
			doc_url_for_function("time::now"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/time")
		);
		assert_eq!(
			doc_url_for_function("count"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/count")
		);
	}

	#[test]
	fn should_resolve_standalone_function_as_namespace() {
		assert_eq!(
			doc_url_for_function("string"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/string")
		);
		assert_eq!(
			doc_url_for_function("array"),
			Some("https://surrealdb.com/docs/surrealql/functions/database/array")
		);
	}

	#[test]
	fn should_return_none_for_unknown() {
		assert_eq!(doc_url("nonexistent"), None);
		assert_eq!(doc_url_for_function("unknown::thing"), None);
	}

	#[test]
	fn should_not_resolve_removed_3x_statements() {
		assert_eq!(doc_url("define scope"), None);
		assert_eq!(doc_url("scope"), None);
		assert_eq!(doc_url("define token"), None);
		assert_eq!(doc_url("token"), None);
	}

	#[test]
	fn should_be_case_insensitive() {
		assert_eq!(doc_url("SELECT"), doc_url("select"));
		assert_eq!(doc_url("Where"), doc_url("where"));
		assert_eq!(
			doc_url_for_function("STRING::LEN"),
			doc_url_for_function("string::len")
		);
	}

	#[test]
	fn should_resolve_top_level_pages() {
		assert_eq!(
			doc_url("operators"),
			Some("https://surrealdb.com/docs/surrealql/operators")
		);
		assert_eq!(
			doc_url("parameters"),
			Some("https://surrealdb.com/docs/surrealql/parameters")
		);
		assert_eq!(
			doc_url("transactions"),
			Some("https://surrealdb.com/docs/surrealql/transactions")
		);
		assert_eq!(
			doc_url("comments"),
			Some("https://surrealdb.com/docs/surrealql/comments")
		);
	}
}
