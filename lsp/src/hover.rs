use tower_lsp::lsp_types::*;

/// Context for a word found in a graph traversal expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GraphContext {
	/// The word is an edge table between arrows: `->edge_table->`
	EdgeTable(String),
	/// The word is the target table after the last arrow: `->target`
	TargetTable(String),
	/// The word is a field on the target table: `->target.field`
	FieldOnTarget { table: String, field: String },
}

/// Extract the full dotted identifier path at the cursor position.
///
/// If the cursor is on `theme` in `settings.theme`, returns `"settings.theme"`.
/// If the cursor is on `settings` in `settings.theme`, returns just `"settings"`.
/// Dots are included as path separators, extending the word leftward through
/// `identifier.identifier` chains.
pub(crate) fn dotted_path_at_position(source: &str, position: Position) -> String {
	let line = match source.split('\n').nth(position.line as usize) {
		Some(l) => l.strip_suffix('\r').unwrap_or(l),
		None => return String::new(),
	};

	let mut utf16_count = 0u32;
	let mut col = line.len();
	for (byte_idx, ch) in line.char_indices() {
		if utf16_count >= position.character {
			col = byte_idx;
			break;
		}
		utf16_count += ch.len_utf16() as u32;
	}

	let bytes = line.as_bytes();
	let is_ident_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

	if col >= bytes.len() || !is_ident_byte(bytes[col]) {
		return String::new();
	}

	// Find the current word boundaries
	let mut word_start = col;
	while word_start > 0 && is_ident_byte(bytes[word_start - 1]) {
		word_start -= 1;
	}
	let mut word_end = col;
	while word_end < bytes.len() && is_ident_byte(bytes[word_end]) {
		word_end += 1;
	}

	// Extend leftward through dot-separated identifier segments
	let mut start = word_start;
	while start >= 2 && bytes[start - 1] == b'.' && is_ident_byte(bytes[start - 2]) {
		let mut seg_start = start - 2;
		while seg_start > 0 && is_ident_byte(bytes[seg_start - 1]) {
			seg_start -= 1;
		}
		start = seg_start;
	}

	if start < word_end {
		line[start..word_end].to_string()
	} else {
		String::new()
	}
}

/// Detect if the word at cursor is preceded by a graph arrow (`->` or `<-`).
///
/// Returns `Some(word)` if the cursor word is immediately after `->` or `<-`,
/// indicating it's a table name in a graph traversal context.
/// Also handles the pattern `->table->target.field` by detecting graph context
/// for any word preceded by `->`.
pub(crate) fn graph_context_at_position(source: &str, position: Position) -> Option<GraphContext> {
	let line = match source.split('\n').nth(position.line as usize) {
		Some(l) => l.strip_suffix('\r').unwrap_or(l),
		None => return None,
	};

	let mut utf16_count = 0u32;
	let mut col = line.len();
	for (byte_idx, ch) in line.char_indices() {
		if utf16_count >= position.character {
			col = byte_idx;
			break;
		}
		utf16_count += ch.len_utf16() as u32;
	}

	let bytes = line.as_bytes();
	let is_ident_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

	// Find the current word boundaries
	let word_start = (0..col)
		.rev()
		.take_while(|&i| is_ident_byte(bytes[i]))
		.last()
		.unwrap_or(col);
	let word_end = (col..bytes.len())
		.take_while(|&i| is_ident_byte(bytes[i]))
		.last()
		.map(|i| i + 1)
		.unwrap_or(col);

	if word_start >= word_end {
		return None;
	}

	let word = &line[word_start..word_end];

	// Check if this word is preceded by a dot (field access on graph target)
	if word_start >= 2 && bytes[word_start - 1] == b'.' {
		let dot_pos = word_start - 1;
		// Find the table word before the dot
		let table_end = dot_pos;
		let table_start = (0..table_end)
			.rev()
			.take_while(|&i| is_ident_byte(bytes[i]))
			.last();
		if let Some(ts) = table_start {
			let table_word = &line[ts..table_end];
			// Check if the table word is preceded by ->
			if ts >= 2 && bytes[ts - 2] == b'-' && bytes[ts - 1] == b'>' {
				return Some(GraphContext::FieldOnTarget {
					table: table_word.to_string(),
					field: word.to_string(),
				});
			}
		}
	}

	// Check if the word is preceded by ->
	if word_start >= 2 && bytes[word_start - 2] == b'-' && bytes[word_start - 1] == b'>' {
		// Check if followed by -> (meaning this is an edge table, not target)
		if word_end + 1 < bytes.len() && bytes[word_end] == b'-' && bytes[word_end + 1] == b'>' {
			return Some(GraphContext::EdgeTable(word.to_string()));
		}
		return Some(GraphContext::TargetTable(word.to_string()));
	}

	// Check if preceded by <-
	if word_start >= 2 && bytes[word_start - 2] == b'<' && bytes[word_start - 1] == b'-' {
		if word_end + 1 < bytes.len() && bytes[word_end] == b'<' && bytes[word_end + 1] == b'-' {
			return Some(GraphContext::EdgeTable(word.to_string()));
		}
		return Some(GraphContext::TargetTable(word.to_string()));
	}

	None
}

pub(crate) fn format_nested_field_hover(
	table_name: &str,
	dotted_path: &str,
	field: &surql_parser::schema_graph::FieldDef,
) -> String {
	let kind = field.kind.as_deref().unwrap_or("any");
	let readonly = if field.readonly { " READONLY" } else { "" };
	let default = field
		.default
		.as_ref()
		.map(|d| format!(" DEFAULT {d}"))
		.unwrap_or_default();
	let comment = field
		.comment
		.as_ref()
		.map(|c| format!("  -- {c}"))
		.unwrap_or_default();
	format!("**{table_name}.{dotted_path}**\n\nType: `{kind}`{default}{readonly}{comment}")
}

pub(crate) fn format_table_hover(
	name: &str,
	table: &surql_parser::schema_graph::TableDef,
	fields: &[surql_parser::schema_graph::FieldDef],
) -> String {
	let schema_type = if table.full {
		"SCHEMAFULL"
	} else {
		"SCHEMALESS"
	};
	let comment_line = table
		.comment
		.as_ref()
		.map(|c| format!("\n\n*{c}*"))
		.unwrap_or_default();
	let field_list = fields
		.iter()
		.map(|f| {
			let kind = f.kind.as_deref().unwrap_or("any");
			let default = f
				.default
				.as_ref()
				.map(|d| format!(" DEFAULT {d}"))
				.unwrap_or_default();
			let readonly = if f.readonly { " READONLY" } else { "" };
			let comment = f
				.comment
				.as_ref()
				.map(|c| format!("  -- {c}"))
				.unwrap_or_default();
			format!("{} : {kind}{default}{readonly}{comment}", f.name)
		})
		.collect::<Vec<_>>()
		.join("\n");
	format!(
		"```surql\n-- TABLE {name} ({schema_type})\n```\n{comment_line}\n\n\
		 ```surql\n{field_list}\n```"
	)
}

pub(crate) fn format_function_hover(func: &surql_parser::schema_graph::FunctionDef) -> String {
	let args = func
		.args
		.iter()
		.map(|(n, t)| format!("{n}: {t}"))
		.collect::<Vec<_>>()
		.join(", ");
	let ret = func
		.returns
		.as_ref()
		.map(|r| format!(" -> {r}"))
		.unwrap_or_default();
	let comment_line = func
		.comment
		.as_ref()
		.map(|c| format!("\n\n*{c}*"))
		.unwrap_or_default();
	format!(
		"**FUNCTION** `fn::{}`{comment_line}\n\n```surql\nfn::{}({args}){ret}\n```",
		func.name, func.name
	)
}

/// SurrealQL type documentation for hover.
pub(crate) fn type_documentation(word: &str, schema: &surql_parser::SchemaGraph) -> Option<String> {
	let lower = word.to_lowercase();
	let doc = match lower.as_str() {
		"string" => {
			"**string** — UTF-8 text\n\n```surql\nDEFINE FIELD name ON user TYPE string\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/strings)"
		}
		"int" => {
			"**int** — 64-bit signed integer\n\n```surql\nDEFINE FIELD age ON user TYPE int\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/numbers)"
		}
		"float" => {
			"**float** — 64-bit floating point\n\n```surql\nDEFINE FIELD score ON user TYPE float\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/numbers)"
		}
		"decimal" => {
			"**decimal** — Arbitrary precision decimal\n\n```surql\nDEFINE FIELD price ON product TYPE decimal\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/numbers)"
		}
		"number" => {
			"**number** — Any numeric type (int, float, or decimal)\n\n```surql\nDEFINE FIELD value ON data TYPE number\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/numbers)"
		}
		"bool" => {
			"**bool** — Boolean (true/false)\n\n```surql\nDEFINE FIELD active ON user TYPE bool DEFAULT true\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/booleans)"
		}
		"datetime" => {
			"**datetime** — ISO 8601 timestamp\n\n```surql\nDEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now()\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/datetimes)"
		}
		"duration" => {
			"**duration** — Time span (e.g., 1h, 30m, 7d)\n\n```surql\nDEFINE FIELD ttl ON cache TYPE duration\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/durations)"
		}
		"object" => {
			"**object** — JSON-like object (key-value map)\n\n```surql\nDEFINE FIELD settings ON user TYPE object DEFAULT {}\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/objects)"
		}
		"array" => {
			"**array** — Ordered collection. Parameterized: `array<string>`\n\n```surql\nDEFINE FIELD tags ON post TYPE array<string> DEFAULT []\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/arrays)"
		}
		"set" => {
			"**set** — Unique collection (no duplicates). Parameterized: `set<string>`\n\n```surql\nDEFINE FIELD roles ON user TYPE set<string>\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/sets)"
		}
		"option" => {
			"**option** — Nullable type. `option<T>` means the field can be NONE\n\n```surql\nDEFINE FIELD bio ON user TYPE option<string>\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/values)"
		}
		"record" => {
			let tables: Vec<_> = schema.table_names().collect();
			let table_list = if tables.is_empty() {
				String::new()
			} else {
				format!(
					"\n\nTables in schema: `{}`",
					tables.into_iter().collect::<Vec<_>>().join("`, `")
				)
			};
			return Some(format!(
				"**record** — Link to another record. Parameterized: `record<table>`\n\n\
				 ```surql\nDEFINE FIELD author ON post TYPE record<user>\n```\n\n\
				 The linked record can be fetched with `FETCH`.{table_list}\n\n\
				 [Docs](https://surrealdb.com/docs/surrealql/datamodel/records)"
			));
		}
		"uuid" => {
			"**uuid** — Universally unique identifier\n\n```surql\nDEFINE FIELD id ON user TYPE uuid DEFAULT rand::uuid::v4()\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/uuid)"
		}
		"bytes" => {
			"**bytes** — Binary data\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/bytes)"
		}
		"geometry" => {
			"**geometry** — GeoJSON geometry (point, line, polygon, etc.)\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/geometries)"
		}
		"any" => {
			"**any** — Accepts any type (no type constraint)\n\n```surql\nDEFINE FIELD data ON flexible_table TYPE any\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/values)"
		}
		_ => return None,
	};
	Some(doc.to_string())
}

/// SurrealQL keyword documentation for hover.
pub(crate) fn keyword_documentation(word: &str) -> Option<&'static str> {
	match word.to_uppercase().as_str() {
		// Compound keywords (checked first via detect_compound_keyword)
		"DEFINE TABLE" => Some(
			"**DEFINE TABLE** — Define a table schema\n\n```surql\nDEFINE TABLE name TYPE NORMAL SCHEMAFULL\nPERMISSIONS FOR select FULL\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/table)",
		),
		"DEFINE FIELD" => Some(
			"**DEFINE FIELD** — Define a field on a table\n\n```surql\nDEFINE FIELD name ON table TYPE string\nDEFAULT 'value' ASSERT $value != NONE\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/field)",
		),
		"DEFINE INDEX" => Some(
			"**DEFINE INDEX** — Create an index on a table\n\n```surql\nDEFINE INDEX name ON table FIELDS field UNIQUE\nDEFINE INDEX name ON table FIELDS field SEARCH ANALYZER\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/indexes)",
		),
		"DEFINE FUNCTION" => Some(
			"**DEFINE FUNCTION** — Define a server-side function\n\n```surql\nDEFINE FUNCTION fn::name($arg: string) -> string {\n\tRETURN 'Hello, ' + $arg;\n};\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/function)",
		),
		"DEFINE EVENT" => Some(
			"**DEFINE EVENT** — Define an event trigger on a table\n\n```surql\nDEFINE EVENT name ON table\nWHEN $event = 'CREATE'\nTHEN { CREATE log SET action = 'created' };\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/event)",
		),
		"DEFINE ANALYZER" => Some(
			"**DEFINE ANALYZER** — Define a search analyzer\n\n```surql\nDEFINE ANALYZER name TOKENIZERS blank, class\nFILTERS lowercase, snowball(english)\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/analyzer)",
		),
		"DEFINE PARAM" => Some(
			"**DEFINE PARAM** — Define a global parameter\n\n```surql\nDEFINE PARAM $name VALUE 'default'\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/param)",
		),
		"DEFINE ACCESS" => Some(
			"**DEFINE ACCESS** — Define authentication access\n\n```surql\nDEFINE ACCESS name ON DATABASE TYPE RECORD\nSIGNUP (CREATE user SET email = $email)\nSIGNIN (SELECT * FROM user WHERE email = $email)\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/access)",
		),
		"DEFINE NAMESPACE" => Some(
			"**DEFINE NAMESPACE** — Create a namespace\n\n```surql\nDEFINE NAMESPACE name\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/namespace)",
		),
		"DEFINE DATABASE" => Some(
			"**DEFINE DATABASE** — Create a database within a namespace\n\n```surql\nDEFINE DATABASE name\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/database)",
		),
		"ORDER BY" => Some(
			"**ORDER BY** — Sort query results\n\n```surql\nSELECT * FROM user ORDER BY name ASC, age DESC\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/clauses/order-by)",
		),
		"GROUP BY" => Some(
			"**GROUP BY** — Group results for aggregation\n\n```surql\nSELECT category, count() FROM post GROUP BY category\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/clauses/group-by)",
		),
		"INSERT INTO" => Some(
			"**INSERT INTO** — Insert records into a table\n\n```surql\nINSERT INTO table { field: value }\nINSERT INTO table [{ ... }, { ... }]\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/insert)",
		),
		"TYPE NORMAL" => Some(
			"**TYPE NORMAL** — Standard table (default)\n\nStores regular records. Use with SCHEMAFULL or SCHEMALESS.\n\n```surql\nDEFINE TABLE user TYPE NORMAL SCHEMAFULL\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/table)",
		),
		"TYPE RELATION" => Some(
			"**TYPE RELATION** — Graph edge table\n\nStores edges between records. Use with RELATE.\n\n```surql\nDEFINE TABLE follows TYPE RELATION FROM user TO user\nRELATE user:alice->follows->user:bob\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/table)",
		),
		"TYPE ANY" => Some(
			"**TYPE ANY** — Accepts both records and relations\n\n```surql\nDEFINE TABLE mixed TYPE ANY SCHEMALESS\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define/table)",
		),
		// Single keywords
		"SELECT" => Some(
			"**SELECT** — Query data from tables\n\n```surql\nSELECT field1, field2 FROM table WHERE condition\nORDER BY field LIMIT n\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/select)",
		),
		"CREATE" => Some(
			"**CREATE** — Create a new record\n\n```surql\nCREATE table SET field = value\nCREATE table CONTENT { field: value }\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/create)",
		),
		"UPDATE" => Some(
			"**UPDATE** — Modify existing records\n\n```surql\nUPDATE table SET field = value WHERE condition\nUPDATE table MERGE { field: value }\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/update)",
		),
		"DELETE" => Some(
			"**DELETE** — Remove records\n\n```surql\nDELETE table WHERE condition\nDELETE record:id\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/delete)",
		),
		"INSERT" => Some(
			"**INSERT** — Insert records (supports ON DUPLICATE KEY UPDATE)\n\n```surql\nINSERT INTO table { field: value }\nINSERT INTO table [{ ... }, { ... }]\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/insert)",
		),
		"UPSERT" => Some(
			"**UPSERT** — Create or update a record atomically\n\n```surql\nUPSERT table SET field = value WHERE condition\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/upsert)",
		),
		"RELATE" => Some(
			"**RELATE** — Create a graph edge between two records\n\n```surql\nRELATE from_record->edge_table->to_record\nSET field = value\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/relate)",
		),
		"DEFINE" => Some(
			"**DEFINE** — Define schema elements\n\n```surql\nDEFINE TABLE name SCHEMAFULL\nDEFINE FIELD name ON table TYPE string\nDEFINE INDEX name ON table FIELDS field UNIQUE\nDEFINE FUNCTION fn::name($arg: type) { ... }\nDEFINE EVENT name ON table WHEN $event = 'CREATE' THEN { ... }\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/define)",
		),
		"REMOVE" => Some(
			"**REMOVE** — Remove schema definitions\n\n```surql\nREMOVE TABLE name\nREMOVE FIELD name ON table\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/remove)",
		),
		"LET" => Some(
			"**LET** — Bind a value to a parameter\n\n```surql\nLET $name = expression\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/let)",
		),
		"IF" => Some(
			"**IF** — Conditional expression\n\n```surql\nIF condition { ... }\nELSE IF condition { ... }\nELSE { ... }\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/ifelse)",
		),
		"FOR" => Some(
			"**FOR** — Iterate over values\n\n```surql\nFOR $item IN $array { ... }\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/for)",
		),
		"RETURN" => Some(
			"**RETURN** — Return a value from a block or function\n\n```surql\nRETURN expression\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/return)",
		),
		"BEGIN" => Some(
			"**BEGIN** — Start a transaction\n\n```surql\nBEGIN TRANSACTION;\n-- statements\nCOMMIT TRANSACTION;\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/begin)",
		),
		"COMMIT" => Some(
			"**COMMIT** — Commit a transaction\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/commit)",
		),
		"CANCEL" => Some(
			"**CANCEL** — Roll back a transaction\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/cancel)",
		),
		"LIVE" => Some(
			"**LIVE** — Subscribe to real-time changes\n\n```surql\nLIVE SELECT * FROM table\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/live)",
		),
		"KILL" => Some(
			"**KILL** — Stop a live query\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/kill)",
		),
		"USE" => Some(
			"**USE** — Switch namespace or database\n\n```surql\nUSE NS namespace DB database\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/use)",
		),
		"INFO" => Some(
			"**INFO** — Show database information\n\n```surql\nINFO FOR DB\nINFO FOR TABLE name\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/info)",
		),
		"SLEEP" => Some(
			"**SLEEP** — Pause execution\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/sleep)",
		),
		"THROW" => Some(
			"**THROW** — Throw a custom error\n\n[Docs](https://surrealdb.com/docs/surrealql/statements/throw)",
		),
		"SCHEMAFULL" => Some("**SCHEMAFULL** — Only allow defined fields on a table"),
		"SCHEMALESS" => Some("**SCHEMALESS** — Allow any fields on a table (default)"),
		"CHANGEFEED" => Some("**CHANGEFEED** — Enable change tracking on a table"),
		"PERMISSIONS" => Some("**PERMISSIONS** — Set access control on tables/fields"),
		"FETCH" => Some("**FETCH** — Eagerly load linked records"),
		"VALUE" | "VALUES" => {
			Some("**VALUE** / **VALUES** — Return raw value instead of wrapped result")
		}
		"EXPLAIN" => Some("**EXPLAIN** — Show query execution plan"),
		"PARALLEL" => Some("**PARALLEL** — Execute query in parallel"),
		"WHERE" => Some("**WHERE** — Filter records by condition"),
		"ORDER" => Some("**ORDER BY** — Sort query results"),
		"GROUP" => Some("**GROUP BY** — Group results for aggregation"),
		"LIMIT" => Some("**LIMIT** — Restrict number of results"),
		"FROM" => Some("**FROM** — Specify the data source"),
		"TABLE" => Some("**TABLE** — Define or reference a table"),
		"FIELD" => Some("**FIELD** — Define a field on a table"),
		"INDEX" => Some("**INDEX** — Create an index on a table"),
		"FUNCTION" => Some("**FUNCTION** — Define a server-side function"),
		"EVENT" => Some("**EVENT** — Define an event trigger on a table"),
		"ANALYZER" => Some("**ANALYZER** — Define a search analyzer"),
		"PARAM" => Some("**PARAM** — Define a global parameter"),
		"ACCESS" => Some("**ACCESS** — Define authentication access"),
		"NAMESPACE" | "NS" => Some("**NAMESPACE** — A top-level organizational unit"),
		"DATABASE" | "DB" => Some("**DATABASE** — A container within a namespace"),
		"SET" => Some("**SET** — Set field values on a record"),
		"CONTENT" => Some("**CONTENT** — Replace all fields with a JSON object"),
		"MERGE" => Some("**MERGE** — Merge fields into existing record"),
		"TYPE" => Some("**TYPE** — Specify field type or table type"),
		"UNIQUE" => Some("**UNIQUE** — Index constraint: no duplicate values"),
		"DEFAULT" => Some("**DEFAULT** — Set a default value for a field"),
		"ASSERT" => Some("**ASSERT** — Validate field value on write"),
		"OVERWRITE" => Some("**OVERWRITE** — Replace existing definition"),
		"FLEXIBLE" => Some("**FLEXIBLE** — Allow any type for a field"),
		"READONLY" => Some("**READONLY** — Field cannot be modified after creation"),
		"ON" => Some("**ON** — Specify the target table for definitions"),
		"IN" => Some("**IN** — Check membership in array or range"),
		"AND" | "OR" => Some("**AND** / **OR** — Logical operators"),
		"NOT" => Some("**NOT** — Logical negation"),
		"AS" => Some("**AS** — Alias for projected fields"),
		"ONLY" => Some("**ONLY** — Return single record instead of array"),
		"SPLIT" => Some("**SPLIT** — Unnest array fields into separate rows"),
		"ELSE" => Some("**ELSE** — Alternate branch of an IF expression"),
		"END" => Some("**END** — Terminate a multi-line statement block"),
		"THEN" => Some("**THEN** — Execute after an event triggers"),
		"WHEN" => Some("**WHEN** — Condition for event triggers"),
		"BREAK" => Some("**BREAK** — Exit a FOR loop early"),
		"CONTINUE" => Some("**CONTINUE** — Skip to next FOR loop iteration"),
		"AFTER" => Some("**AFTER** — Return record state after modification"),
		"BEFORE" => Some("**BEFORE** — Return record state before modification"),
		"DIFF" => Some("**DIFF** — Return JSON patch diff of changes"),
		"TIMEOUT" => Some("**TIMEOUT** — Set maximum execution time"),
		"INTO" => Some("**INTO** — Specify target table for INSERT"),
		"BY" => Some("**BY** — Used with GROUP BY, ORDER BY"),
		"TO" => Some("**TO** — Specify end of relation or cast target"),
		"OMIT" => Some("**OMIT** — Exclude specific fields from results"),
		"START" => Some("**START** — Skip results (offset)"),
		"SHOW" => Some("**SHOW** — Display change feed entries"),
		"NORMAL" => Some("**TYPE NORMAL** — Standard table (default)"),
		"RELATION" => Some("**TYPE RELATION** — Graph edge table"),
		"DROP" => Some("**DROP** — Auto-delete old records"),
		"CONTAINS" => Some("**CONTAINS** — Check if array/string contains a value"),
		_ => None,
	}
}
