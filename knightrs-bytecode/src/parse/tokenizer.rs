use super::{Origin, Result, Span, Stream, Token, TokenKind};
use crate::{strings::KnStr, Environment, Options};
use std::{mem::MaybeUninit, str::CharIndices};

#[cfg(feature = "extensions")]
pub type TokenizerFn<'src> = dyn FnMut(&mut Stream<'src>) -> Result<'src, Token<'src>>;

pub trait ParseToken<'src> {
	fn parse_token(tokenizer: &mut Tokenizer<'src, '_, '_>) -> Result<'src, Option<Token<'src>>>;
}

pub struct Tokenizer<'src, 'env, 'gc> {
	stream: Stream<'src>,
	env: &'env mut Environment<'gc>,
}

impl<'src, 'env, 'gc> Tokenizer<'src, 'env, 'gc> {
	/// Create a new [`Tokenizer`] with the given
	#[inline]
	pub fn new(stream: Stream<'src>, env: &'env mut Environment<'gc>) -> Self {
		Self { stream, env }
	}

	/// Gets the [`Environment`] behind the tokenizer.
	#[inline]
	pub fn env(&self) -> &&'env mut Environment<'gc> {
		&self.env
	}

	/// Gets the [`Stream`] that the tokenizer is using.
	pub fn stream(&mut self) -> &mut Stream<'src> {
		&mut self.stream
	}

	/// Remove all excess whitespace and comments from the start of the string.
	pub fn strip_whitespace_and_comments(&mut self) {
		loop {
			self.stream.take_while(|chr| {
				#[cfg(not(feature = "check-parens"))]
				if matches!(chr, ':' | '(' | ')') {
					return true;
				}

				// We only check for ascii whitespace
				if cfg!(feature = "utf8-strings") {
					chr.is_whitespace()
				} else {
					chr.is_ascii_whitespace()
				}
			});

			if self.stream.peek() == Some('#') {
				self.stream.take_while(|chr| chr != '\n');
			} else {
				break;
			}
		}
	}
}

impl<'src> Iterator for Tokenizer<'src, '_, '_> {
	type Item = Result<'src, Token<'src>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.strip_whitespace_and_comments();
		const PARSE_FNS: [Option<u8>; 255] = {
			let mut arr = [None; 255];
			arr[b'0' as usize] = Some(1);
			// arr[b'0' as usize..=b'9' as usize].fill(Some(1));
			// ...
			arr
		};

		// const PARSE_FNS: [Option<fn(...) -> ...>; 255] = [
		// 	[b'0'..=b'9']: Some(parse_integer),
		// 	[b'a'..=b'z']: Some(parse_variable),
		// 	[b'_']: Some(parse_variable),
		// 	else: NOne
		// ];

		if let Some(digits) = self.stream().take_while(|c| c.is_ascii_digit()) {
			// TODO: float extensions
			return Some(Ok(Token { span: digits, kind: TokenKind::Integer }));
		};

		// if self.
		// self.0.next()
		todo!()
	}
}
