use super::{ParseError, ParseErrorKind};

#[derive(Debug)]
pub struct Stream<'a> {
	line: usize,
	source: &'a str
}

impl<'a> Stream<'a> {
	pub const fn new(source: &'a str) -> Self {
		Self { line: 1, source }
	}

	pub const fn line(&self) -> usize {
		self.line
	}

	pub const fn error(&self, kind: ParseErrorKind) -> ParseError {
		ParseError { line: self.line(), kind }
	}
}

impl Iterator for Stream<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
	}
}