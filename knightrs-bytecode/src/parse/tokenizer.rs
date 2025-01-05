use super::{Origin, Span, Stream, Token};
use crate::strings::KnStr;
use std::str::CharIndices;

pub struct Tokenizer<'src> {
	stream: Stream<'src>,
	other_tokenizers: Vec<Box<dyn FnMut(Stream<'src>) -> Option<Token<'src>>>>,
}

impl<'src> Iterator for Tokenizer<'src> {
	type Item = Result;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
impl Tokenizer<'src>

// 	source: &'src KnStr,

// 	chars: CharIndices<'src>,
// }

// impl<'src> Stream<'src> {
// 	/// Create a new [`Stream`].
// 	pub fn new(source: &'src KnStr, origin: Origin<'src>) -> Self {
// 		Self { source, origin, chars: source.as_str().char_indices() }
// 	}

// 	/// Look at the next character without consuming it.
// 	#[must_use]
// 	pub fn peek(&self) -> Option<char> {
// 		self.chars.clone().next().map(|(_, chr)| chr)
// 	}

// 	/// Consume the next character and return it.
// 	pub fn advance(&mut self) -> Option<char> {
// 		self.chars.next().map(|(_, chr)| chr)
// 	}

// 	/// Return the _entire_ program's source
// 	pub fn source(&self) -> &'src KnStr {
// 		self.source
// 	}

// 	// TODO: take_if, if needed

// 	/// [`advance`]s while `cond` is true, returning a [`Span`] of the advanced characters if the
// 	/// condition was true at least once.
// 	pub fn take_while(&mut self, mut cond: impl FnMut(char) -> bool) -> Option<Span<'src>> {
// 		let start = self.chars.clone();

// 		while self.chars.next().map_or(false, |(_, chr)| cond(chr)) {
// 			// do nothing, we're advancing
// 		}

// 		// OPTIMIZE ME: this could be optimized more if we did eventually use `unsafe`.
// 		let snippet = &self.source.as_str()[start.offset()..self.chars.offset()];
// 		let snippet = KnStr::new_unvalidated(snippet);

// 		if snippet.is_empty() {
// 			None
// 		} else {
// 			Some(Span { snippet, origin: self.origin })
// 		}
// 	}
// }
