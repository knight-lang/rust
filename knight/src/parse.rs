use crate::{Environment, Value};

mod functions;
mod stream;
mod error;
pub mod tokenize;

pub use error::{ParseError, ParseErrorKind, ParseResult};
pub use functions::Functions;
pub use stream::Stream;
pub use tokenize::TokenizeFn;

cfg_if! {
	if #[cfg(feature="disallow-unicode")] {
		pub type Character = u8;
	} else {
		pub type Character = char;
	}
}

pub struct Parser<'env, 'stream> {
	env: &'env Environment,
	stream: Stream<'stream>
}

impl<'env> Parser<'env, '_> {
	pub fn next_value(&mut self) -> ParseResult<Value<'env>> {
		let prefix = self.next_character().ok_or_else(|| self.stream.error(ParseErrorKind::NothingToBeParsed))?;

		let func = self.env.functions().get(prefix).ok_or_else(|| self.stream.error(ParseErrorKind::UnknownTokenStart))?;

		(func)(self, prefix)
	}

	pub fn next_character(&mut self) -> Option<Character> {
		self.stream.next()
	}
}
