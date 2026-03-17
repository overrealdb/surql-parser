//! Compatibility stubs for types referenced by the upstream parser code.
//!
//! The SurrealDB parser references several engine-internal types.
//! This module provides minimal stubs or re-exports to satisfy compilation.
//!
//! These stubs are stable — they only change when SurrealDB adds new types
//! to the parser AST, which is rare. When it happens, CI fails with
//! "cannot find type X" and you add one line here.

// ─── types (Public* aliases from surrealdb-core) ───

/// Re-exports from surrealdb-types with the Public* aliases that surrealdb-core uses.
pub mod types {
	pub use surrealdb_types::{
		Array as PublicArray, Bytes as PublicBytes, Datetime as PublicDatetime,
		Duration as PublicDuration, File as PublicFile, Geometry as PublicGeometry,
		GeometryKind as PublicGeometryKind, Kind as PublicKind, KindLiteral as PublicKindLiteral,
		Number as PublicNumber, Object as PublicObject, Range as PublicRange,
		RecordId as PublicRecordId, RecordIdKey as PublicRecordIdKey,
		RecordIdKeyRange as PublicRecordIdKeyRange, Regex as PublicRegex, Set as PublicSet,
		Table as PublicTable, Uuid as PublicUuid, Value as PublicValue,
	};
}

// ─── val (crate::val::* paths) ───

/// Re-exports from surrealdb-types matching crate::val::* paths in upstream code.
pub mod val {
	pub use surrealdb_types::GeometryKind;
	pub use surrealdb_types::{
		Array, Bytes, Datetime, Decimal, Duration, File, Geometry, Number, Object, Range, RecordId,
		RecordIdKey, RecordIdKeyRange, Regex, Set, Table, Uuid, Value,
	};

	/// TableName newtype
	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	pub struct TableName(pub String);

	impl TableName {
		pub fn new(s: impl Into<String>) -> Self {
			Self(s.into())
		}
	}

	impl std::fmt::Display for TableName {
		fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
			write!(f, "{}", self.0)
		}
	}

	/// Decimal extension trait
	pub trait DecimalExt {
		fn is_integer(&self) -> bool;
	}

	impl DecimalExt for Decimal {
		fn is_integer(&self) -> bool {
			self.fract().is_zero()
		}
	}

	pub mod duration {
		pub use surrealdb_types::Duration;
		pub const SECONDS_PER_MINUTE: u64 = 60;
		pub const SECONDS_PER_HOUR: u64 = 3600;
		pub const SECONDS_PER_DAY: u64 = 86400;
		pub const SECONDS_PER_WEEK: u64 = 604800;
		pub const SECONDS_PER_YEAR: u64 = 31536000;
	}

	pub mod range {
		pub use surrealdb_types::{Range, RecordIdKey, RecordIdKeyRange};

		/// TypedRange used in mock.rs
		#[derive(Debug, Clone, PartialEq, Eq, Hash)]
		pub struct TypedRange<T> {
			pub start: std::ops::Bound<T>,
			pub end: std::ops::Bound<T>,
		}

		impl<T: PartialOrd> PartialOrd for TypedRange<T> {
			fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
				None // Range comparison not meaningful
			}
		}
	}
}

// ─── catalog (schema definition types) ───

/// Stubs for catalog types used in parser AST (DEFINE TABLE/INDEX/EVENT etc.)
///
/// These are schema-level type definitions — they rarely change.
/// If SurrealDB adds a new variant, CI will fail and you add it here.
pub mod catalog {
	use serde::{Deserialize, Serialize};

	#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
	pub enum EventKind {
		Create,
		Update,
		Delete,
		Sync,
		Async { retry: u32, max_depth: u32 },
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub enum ApiMethod {
		Get,
		Post,
		Put,
		Patch,
		Delete,
		Trace,
	}

	impl surrealdb_types::ToSql for ApiMethod {
		fn fmt_sql(&self, f: &mut String, _fmt: surrealdb_types::SqlFormat) {
			f.push_str(match self {
				Self::Get => "GET",
				Self::Post => "POST",
				Self::Put => "PUT",
				Self::Patch => "PATCH",
				Self::Delete => "DELETE",
				Self::Trace => "TRACE",
			});
		}
	}

	#[derive(Debug, Clone, PartialEq)]
	pub enum Permission {
		None,
		Full,
		Specific(Box<crate::upstream::sql::expression::Expr>),
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct Permissions {
		pub select: Permission,
		pub create: Permission,
		pub update: Permission,
		pub delete: Permission,
	}

	impl Default for Permissions {
		fn default() -> Self {
			Self {
				select: Permission::Full,
				create: Permission::Full,
				update: Permission::Full,
				delete: Permission::Full,
			}
		}
	}

	#[derive(Debug, Clone, PartialEq)]
	pub enum Index {
		Idx,
		Uniq,
		Count(Option<crate::upstream::sql::Cond>),
		FullText(FullTextParams),
		Hnsw(HnswParams),
	}

	// Conversion impls needed by upstream From<sql::Index> for catalog::Index
	impl From<crate::upstream::sql::Cond> for crate::upstream::sql::expression::Expr {
		fn from(c: crate::upstream::sql::Cond) -> Self {
			c.0
		}
	}

	impl From<crate::upstream::sql::expression::Expr> for crate::upstream::sql::Cond {
		fn from(e: crate::upstream::sql::expression::Expr) -> Self {
			crate::upstream::sql::Cond(e)
		}
	}

	impl From<Box<crate::upstream::sql::expression::Expr>> for crate::upstream::sql::expression::Expr {
		fn from(b: Box<crate::upstream::sql::expression::Expr>) -> Self {
			*b
		}
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct FullTextParams {
		pub analyzer: String,
		pub highlight: bool,
		pub scoring: Scoring,
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct HnswParams {
		pub dimension: u16,
		pub distance: Distance,
		pub vector_type: VectorType,
		pub m: u8,
		pub m0: u8,
		pub ml: crate::compat::types::PublicNumber,
		pub ef_construction: u16,
		pub extend_candidates: bool,
		pub keep_pruned_connections: bool,
		pub use_hashed_vector: bool,
	}

	#[derive(Debug, Clone, Copy, PartialEq)]
	pub enum Distance {
		Chebyshev,
		Cosine,
		Euclidean,
		Hamming,
		Jaccard,
		Manhattan,
		Minkowski(crate::compat::types::PublicNumber),
		Pearson,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
	pub enum VectorType {
		F32,
		F64,
		I16,
		I32,
		I64,
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum Scoring {
		Bm { k1: f32, b: f32 },
		Vs,
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum TableType {
		Any,
		Normal,
		Relation(Relation),
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub struct Relation {
		pub from: Vec<String>,
		pub to: Vec<String>,
		pub enforced: bool,
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum ModuleName {
		Module(String),
		Silo(String, String),
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub struct GraphQLConfig {
		pub tables: GraphQLTablesConfig,
		pub functions: GraphQLFunctionsConfig,
		pub introspection: GraphQLIntrospectionConfig,
		pub depth_limit: Option<u32>,
		pub complexity_limit: Option<u32>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum GraphQLTablesConfig {
		None,
		Auto,
		Include(Vec<String>),
		Exclude(Vec<String>),
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum GraphQLFunctionsConfig {
		None,
		Auto,
		Include(Vec<String>),
		Exclude(Vec<String>),
	}

	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum GraphQLIntrospectionConfig {
		None,
		Auto,
	}

	/// EventDefinition used in DEFINE EVENT parsing
	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub struct EventDefinition {
		pub kind: EventKind,
	}

	impl EventDefinition {
		pub const DEFAULT_RETRY: u32 = 3;
		pub const DEFAULT_MAX_DEPTH: u32 = 10;
	}
}

// ─── engine stubs ───

/// Minimal Capabilities stub (parser only needs feature flags).
pub struct Capabilities {
	files_enabled: bool,
	surrealism_enabled: bool,
}

impl Capabilities {
	pub fn all() -> Self {
		Self {
			files_enabled: true,
			surrealism_enabled: true,
		}
	}

	pub fn allows_experimental(&self, target: &ExperimentalTarget) -> bool {
		match target {
			ExperimentalTarget::Files => self.files_enabled,
			ExperimentalTarget::Surrealism => self.surrealism_enabled,
		}
	}
}

pub mod capabilities {
	pub enum ExperimentalTarget {
		Files,
		Surrealism,
	}
}
pub use capabilities::ExperimentalTarget;

/// Minimal error stub
pub mod err {
	#[derive(Debug, thiserror::Error)]
	pub enum Error {
		#[error("Query too large")]
		QueryTooLarge,
		#[error("Invalid query: {0}")]
		InvalidQuery(crate::upstream::syn::error::RenderedError),
		#[error("Access grant bearer invalid")]
		AccessGrantBearerInvalid,
	}
}

/// Stub for dbs types
pub mod dbs {}

/// Display impl for Param (needed because ToSql doesn't provide Display)
impl std::fmt::Display for crate::upstream::sql::Param {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		use surrealdb_types::{SqlFormat, ToSql};
		let mut buf = String::new();
		self.fmt_sql(&mut buf, SqlFormat::SingleLine);
		f.write_str(&buf)
	}
}

/// Free function replacement for Decimal::from_str_normalized (not in surrealdb-types 3.0.4)
pub fn decimal_from_str_normalized(s: &str) -> Result<rust_decimal::Decimal, rust_decimal::Error> {
	use std::str::FromStr;
	let d = rust_decimal::Decimal::from_str(s)?;
	Ok(d.normalize())
}

/// fmt helpers not yet in surrealdb-types 3.0 (added in 3.1)
pub mod fmt {
	pub const fn fmt_non_finite_f64(v: f64) -> Option<&'static str> {
		match v {
			f64::INFINITY => Some("Infinity"),
			f64::NEG_INFINITY => Some("-Infinity"),
			_ if v.is_nan() => Some("NaN"),
			_ => None,
		}
	}
}
