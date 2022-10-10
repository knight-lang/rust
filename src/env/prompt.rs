use super::{Environment, Flags};
use crate::value::Text;
use crate::Result;
use std::io::{self, BufRead};

#[cfg(feature = "extensions")]
use {
	crate::value::{text::TextSlice, Runnable, ToText, Value},
	crate::Ast,
	std::collections::VecDeque,
};

pub trait Stdin: BufRead + crate::containers::MaybeSendSync {}
impl<T: BufRead + crate::containers::MaybeSendSync> Stdin for T {}

pub struct Prompt<'e> {
	default: Box<dyn Stdin + 'e>,

	#[cfg(feature = "extensions")]
	replacement: Option<PromptReplacement<'e>>,
}

impl Default for Prompt<'_> {
	fn default() -> Self {
		Self {
			default: Box::new(io::BufReader::new(io::stdin())),

			#[cfg(feature = "extensions")]
			replacement: None,
		}
	}
}

#[cfg(feature = "extensions")]
enum PromptReplacement<'e> {
	Closed,
	Buffered(VecDeque<Text>),
	Computed(Ast<'e>),
}

fn strip_ending(line: &mut String) {
	match line.pop() {
		Some('\n') => {}
		Some('\r') => {}
		Some(other) => {
			line.push(other);
			return;
		}
		None => return,
	}

	loop {
		match line.pop() {
			Some('\r') => {}
			Some(other) => {
				line.push(other);
				return;
			}
			None => return,
		}
	}
}

pub struct Line<'e>(Option<ReadLineResultInner<'e>>);
enum ReadLineResultInner<'e> {
	Text(Text),

	#[allow(unused)]
	#[cfg(not(feature = "extensions"))]
	_Never(std::marker::PhantomData<&'e ()>),

	#[cfg(feature = "extensions")]
	Ast(Ast<'e>),
}

impl<'e> Line<'e> {
	#[inline]
	pub fn get(self, env: &mut Environment<'e>) -> Result<Option<Text>> {
		match self.0 {
			None => Ok(None),
			Some(ReadLineResultInner::Text(text)) => Ok(Some(text)),

			#[cfg(not(feature = "extensions"))]
			Some(ReadLineResultInner::_Never(_)) => {
				let _ = env;
				unreachable!()
			}

			#[cfg(feature = "extensions")]
			Some(ReadLineResultInner::Ast(ast)) => match ast.run(env)? {
				Value::Null => Ok(None),
				other => other.to_text(env).map(Some),
			},
		}
	}
}

impl<'e> Prompt<'e> {
	/// Sets the default stdin.
	///
	/// This doesn't affect any replacements that may be enabled.
	pub fn set_stdin<S: Stdin + 'e>(&mut self, stdin: S) {
		self.default = Box::new(stdin);
	}

	/// Reads a line from stdin.
	///
	/// Instead of directly returning the [`Text`] line, this instead returns the [`Line`] type. You
	/// must then call [`.get(env)`](Line::get) on it to get the actual [`Text`].
	///
	/// # Errors
	/// Any errors that occur when reading from stdin are bubbled upwards.
	#[cfg_attr(not(feature = "extensions"), inline)]
	pub fn read_line(&mut self, flags: &Flags) -> Result<Line<'e>> {
		#[cfg(feature = "extensions")]
		match self.replacement.as_mut() {
			Some(PromptReplacement::Closed) => return Ok(Line(None)),
			Some(PromptReplacement::Buffered(queue)) => {
				return Ok(Line(queue.pop_front().map(ReadLineResultInner::Text)))
			}
			Some(PromptReplacement::Computed(ast)) => {
				return Ok(Line(Some(ReadLineResultInner::Ast(ast.clone()))))
			}
			None => {}
		}

		let mut line = String::new();

		// If we read an empty line, return null.
		if self.default.read_line(&mut line)? == 0 {
			return Ok(Line(None));
		}

		strip_ending(&mut line);
		Ok(Line(Some(ReadLineResultInner::Text(Text::new(line, flags)?))))
	}
}

#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
/// Replacement-related functions.
///
/// If
impl<'e> Prompt<'e> {
	/// Clears the currently set replacement, if any.
	pub fn reset_replacement(&mut self) {
		self.replacement = None;
	}

	/// Mimics stdin reaching EOF.
	///
	/// This clears any previous replacement.
	pub fn close(&mut self) {
		self.replacement = Some(PromptReplacement::Closed);
	}

	/// Calling `PROMPT` will actually run `ast` and convert its return value to a [`Text`].
	///
	/// This clears any previous replacement.
	pub fn set_ast(&mut self, ast: Ast<'e>) {
		self.replacement = Some(PromptReplacement::Computed(ast));
	}

	/// Adds `new_lines` to a queue of lines to be returned when `PROMPT` is called. If
	///
	/// This will clear any previous [`close()`](Self::close) and [`set_ast()`](Self::set_ast)
	/// replacements. However, it will _not_ clear previous `add_lines` replacements, and instead
	/// will simply add `new_lines` to the end.
	pub fn add_lines(&mut self, new_lines: &TextSlice) {
		let lines = match self.replacement {
			Some(PromptReplacement::Buffered(ref mut lines)) => lines,
			_ => {
				self.replacement = Some(PromptReplacement::Buffered(Default::default()));
				match self.replacement {
					Some(PromptReplacement::Buffered(ref mut lines)) => lines,
					_ => unreachable!(),
				}
			}
		};

		for line in (**new_lines).split('\n') {
			let mut line = line.to_string();
			strip_ending(&mut line);
			lines.push_back(unsafe { Text::new_unchecked(line) });
		}
	}
}
