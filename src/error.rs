//! Custom error types for the surql-parser public API.

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Parse error: {0}")]
	Parse(String),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Schema error: {0}")]
	Schema(String),

	#[error("Format error: {0}")]
	Format(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_display_parse_error() {
		let err = Error::Parse("unexpected token".into());
		assert_eq!(err.to_string(), "Parse error: unexpected token");
	}

	#[test]
	fn should_display_schema_error() {
		let err = Error::Schema("table not found".into());
		assert_eq!(err.to_string(), "Schema error: table not found");
	}

	#[test]
	fn should_display_format_error() {
		let err = Error::Format("invalid indent".into());
		assert_eq!(err.to_string(), "Format error: invalid indent");
	}

	#[test]
	fn should_convert_from_io_error() {
		let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
		let err: Error = io_err.into();
		assert!(matches!(err, Error::Io(_)));
		assert!(err.to_string().contains("file not found"));
	}

	#[test]
	fn should_return_parse_error_from_invalid_surql() {
		let result = crate::parse("SELEC * FORM user");
		assert!(result.is_err());
		let err = result.unwrap_err();
		assert!(matches!(err, Error::Parse(_)));
	}

	#[test]
	fn should_return_ok_from_valid_surql() {
		let result = crate::parse("SELECT * FROM user");
		assert!(result.is_ok());
	}
}
