use crate::upstream::fmt::EscapeKwFreeIdent;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Model {
	pub name: String,
	pub version: String,
}
impl ToSql for Model {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		f.push_str("ml");
		for s in self.name.split("::") {
			f.push_str("::");
			write_sql!(f, fmt, "{}", EscapeKwFreeIdent(s));
		}
		write_sql!(f, fmt, "<{}>", self.version);
	}
}
