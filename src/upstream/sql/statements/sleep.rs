use crate::compat::types::PublicDuration;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Hash)]
pub struct SleepStatement {
	pub duration: PublicDuration,
}
impl ToSql for SleepStatement {
	fn fmt_sql(&self, f: &mut String, sql_fmt: SqlFormat) {
		write_sql!(f, sql_fmt, "SLEEP {}", self.duration);
	}
}
