//! Module for rendering errors onto source code.
use super::{Location, MessageKind};
use std::cmp::Ordering;
use std::fmt;
use std::ops::Range;
#[derive(Clone, Debug)]
pub struct RenderedError {
	pub errors: Vec<String>,
	pub snippets: Vec<Snippet>,
}
impl RenderedError {
	/// Offset the snippet locations within the rendered error by a given number
	/// of lines and columns.
	///
	/// The column offset is only applied to the any snippet which is at line 1
	pub fn offset_location(mut self, line: usize, col: usize) -> Self {
		for s in self.snippets.iter_mut() {
			if s.location.line == 1 {
				s.location.column += col;
			}
			s.location.line += line;
		}
		self
	}
}
impl fmt::Display for RenderedError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.errors.len().cmp(&1) {
			Ordering::Equal => writeln!(f, "{}", self.errors[0])?,
			Ordering::Greater => {
				writeln!(f, "- {}", self.errors[0])?;
				writeln!(f, "caused by:")?;
				for e in &self.errors[2..] {
					writeln!(f, "    - {}", e)?
				}
			}
			Ordering::Less => {}
		}
		for s in &self.snippets {
			writeln!(f, "{s}")?;
		}
		Ok(())
	}
}
/// Whether the snippet was truncated.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Truncation {
	/// The snippet wasn't truncated
	None,
	/// The snippet was truncated at the start
	Start,
	/// The snippet was truncated at the end
	End,
	/// Both sided of the snippet where truncated.
	Both,
}
/// A piece of the source code with a location and an optional explanation.
#[derive(Clone, Debug)]
pub struct Snippet {
	/// The part of the original source code,
	source: String,
	/// Whether part of the source line was truncated.
	truncation: Truncation,
	/// The location of the snippet in the original source code.
	location: Location,
	/// The offset, in chars, into the snippet where the location is.
	offset: usize,
	/// The amount of characters that are part of area to be pointed to.
	length: usize,
	/// A possible explanation for this snippet.
	label: Option<String>,
	/// The kind of snippet,
	#[expect(dead_code)]
	kind: MessageKind,
}
impl Snippet {
	/// How long with the source line have to be before it gets truncated.
	const MAX_SOURCE_DISPLAY_LEN: usize = 80;
	/// How far the will have to be in the source line before everything before
	/// it gets truncated.
	const MAX_ERROR_LINE_OFFSET: usize = 50;
	pub fn from_source_location(
		source: &str,
		location: Location,
		explain: Option<&'static str>,
		kind: MessageKind,
	) -> Self {
		let line = source
			.split('\n')
			.nth(location.line - 1)
			.expect("line exists in source");
		let (line, truncation, offset) = Self::truncate_line(line, location.column - 1);
		Snippet {
			source: line.to_owned(),
			truncation,
			location,
			offset,
			length: 1,
			label: explain.map(|x| x.into()),
			kind,
		}
	}
	pub fn from_source_location_range(
		source: &str,
		location: Range<Location>,
		explain: Option<&str>,
		kind: MessageKind,
	) -> Self {
		let line = source
			.split('\n')
			.nth(location.start.line - 1)
			.expect("line exists in source");
		let (line, truncation, offset) = Self::truncate_line(line, location.start.column - 1);
		let length = if location.start.line == location.end.line {
			(location.end.column - location.start.column).max(1)
		} else {
			1
		};
		Snippet {
			source: line.to_owned(),
			truncation,
			location: location.start,
			offset,
			length,
			label: explain.map(|x| x.into()),
			kind,
		}
	}
	/// Trims whitespace of an line and additionally truncates the string around
	/// the target_col_offset if it is too long.
	///
	/// returns the trimmed string, how it is truncated, and the offset into
	/// truncated the string where the target_col is located.
	fn truncate_line(mut line: &str, target_col: usize) -> (&str, Truncation, usize) {
		let mut offset = 0;
		for (i, (idx, c)) in line.char_indices().enumerate() {
			if i == target_col || !c.is_whitespace() {
				line = &line[idx..];
				offset = target_col - i;
				break;
			}
		}
		line = line.trim_end();
		let mut truncation = Truncation::None;
		if offset > Self::MAX_ERROR_LINE_OFFSET {
			let too_much_offset = offset - 10;
			let mut chars = line.chars();
			for _ in 0..too_much_offset {
				chars.next();
			}
			offset = 10;
			line = chars.as_str();
			truncation = Truncation::Start;
		}
		if line.chars().count() > Self::MAX_SOURCE_DISPLAY_LEN {
			let mut size = Self::MAX_SOURCE_DISPLAY_LEN - 3;
			if truncation == Truncation::Start {
				truncation = Truncation::Both;
				size -= 3;
			} else {
				truncation = Truncation::End
			}
			let truncate_index = line
				.char_indices()
				.nth(size)
				.expect("character index exists")
				.0;
			line = &line[..truncate_index];
		}
		(line, truncation, offset)
	}
}
impl fmt::Display for Snippet {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let spacing = self.location.line.ilog10() as usize + 1;
		for _ in 0..spacing {
			f.write_str(" ")?;
		}
		writeln!(f, "--> [{}:{}]", self.location.line, self.location.column)?;
		for _ in 0..spacing {
			f.write_str(" ")?;
		}
		f.write_str(" |\n")?;
		write!(f, "{:>spacing$} | ", self.location.line)?;
		match self.truncation {
			Truncation::None => {
				writeln!(f, "{}", self.source)?;
			}
			Truncation::Start => {
				writeln!(f, "...{}", self.source)?;
			}
			Truncation::End => {
				writeln!(f, "{}...", self.source)?;
			}
			Truncation::Both => {
				writeln!(f, "...{}...", self.source)?;
			}
		}
		let error_offset = self.offset
			+ if matches!(self.truncation, Truncation::Start | Truncation::Both) {
				3
			} else {
				0
			};
		for _ in 0..spacing {
			f.write_str(" ")?;
		}
		f.write_str(" | ")?;
		for _ in 0..error_offset {
			f.write_str(" ")?;
		}
		for _ in 0..self.length {
			write!(f, "^")?;
		}
		if let Some(ref explain) = self.label {
			write!(f, " {explain}")?;
		}
		Ok(())
	}
}
