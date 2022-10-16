//! How Knight reads from stdin.

use super::{Environment, Flags};
use crate::containers::MaybeSendSync;
use crate::value::integer::IntType;
use crate::value::text::{Encoding, Text};
use crate::Result;
use std::io::{self, BufRead};
use std::marker::PhantomData;

#[cfg(feature = "extensions")]
use {
	crate::value::{text::TextSlice, Runnable, ToText, Value},
	crate::Ast,
	std::collections::VecDeque,
};

/// A trait used for reading from stdin.
///
/// This exists instead of simply using [`BufRead`] because we only need `Send + Sync` when the
/// `multithreaded` feature is enabled, but we want a uniform interface.
pub trait Stdin: BufRead + MaybeSendSync {}
impl<T: BufRead + MaybeSendSync> Stdin for T {}

/// The type that's in charge of reading lines from stdin.
///
/// # Replacements
/// Enabling `extensions` allows you to use replacements: the ability to change what `PROMPT` will
/// return from within Knight itself. Only a single replacement can be in use at a time (i.e.
/// setting a new one will override the previous one) and there's three versions:
///
/// - **end of file**: Acts as if stdin is at end of file. Set via [`Prompt::eof`].
/// - **buffered**: Specify the lines that future invocations of `PROMPT` will return. Once the
///    buffer of lines is empty, acts like **end of file**. Set via [`Prompt::add_lines`].
/// - **computed**: Executes an [`Ast`](crate::Ast) each time `PROMPT` is called. Set via
///   [`Prompt::set_ast`].
///
/// If the [assign to prompt](crate::env::flags::AssignTo::prompt) flag is enabled, you can set
/// replacements from within Knight:
///
/// ```knight
/// # Pretends like prompt is at EOF.
/// ; = PROMPT NULL # You can use `FALSE` as an alias of `NULL` .
/// ; DUMP PROMPT #=> null
///
/// # Specify what future results of `PROMPT` will return.
/// #
/// # If the string has multiple lines, each line is returned separately.
/// ; = PROMPT "hello"
/// ; = PROMPT "world
/// !"
/// ; DUMP PROMPT #=> "hello"
/// ; DUMP PROMPT #=> "world"
/// ; DUMP PROMPT #=> "!"
/// ; DUMP PROMPT #=> null
///
/// # Dynamically compute what `PROMPT` returns.
/// #
/// # If the `BLOCK` returns `NULL`, it acts like `PROMPT` is at
/// # eof. Anything else is converted to a Text before being returned.
/// #
/// ; = lineno 0
/// : = PROMPT BLOCK
///    : IF (> lineno 3)
///      : NULL
///    : = lineno + lineno 1
/// ; DUMP PROMPT #=> "1"
/// ; DUMP PROMPT #=> "2"
/// ; DUMP PROMPT #=> "3"
/// ; DUMP PROMPT #=> null
///
/// # Resets all replacements, so `PROMPT` behaves normally.
/// ; = PROMPT TRUE
/// : DUMP PROMPT #=> reads from stdin
/// ```
pub struct Prompt<'e, I, E> {
	default: Box<dyn Stdin + 'e>,
	_pd: PhantomData<(I, E)>,
	flags: &'e Flags,

	#[cfg(feature = "extensions")]
	replacement: Option<PromptReplacement<I, E>>,
}

#[cfg(feature = "extensions")]
enum PromptReplacement<I, E> {
	Eof,
	Buffered(VecDeque<Text<E>>),
	Computed(Ast<I, E>),
}

fn strip_ending(line: &mut String) {
	match line.pop() {
		Some('\n' | '\r') => {}
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

/// Represents a line read from stdin.
///
/// See [`Prompt::read_line`] for details.
pub struct Line<I, E>(Option<ReadLineResultInner<I, E>>);
enum ReadLineResultInner<I, E> {
	Text(Text<E>),

	#[allow(dead_code)]
	#[cfg(not(feature = "extensions"))]
	_Never(PhantomData<(I, E)>),

	#[cfg(feature = "extensions")]
	Ast(Ast<I, E>),
}

impl<I: IntType, E: Encoding> Line<I, E> {
	/// Gets the `Text` corresponding to this line. Returns `None` if at eof.
	pub fn get(self, env: &mut Environment<I, E>) -> Result<Option<Text<E>>> {
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

impl<'e, I, E> Prompt<'e, I, E> {
	pub(super) fn new(flags: &'e Flags) -> Self {
		Self {
			default: Box::new(io::BufReader::new(io::stdin())),
			flags,
			_pd: PhantomData,

			#[cfg(feature = "extensions")]
			replacement: None,
		}
	}
	/// Sets the default stdin.
	///
	/// This doesn't affect any replacements that may have been set.
	pub fn set_stdin<S: Stdin + 'e>(&mut self, stdin: S) {
		self.default = Box::new(stdin);
	}

	/// Reads a line from stdin.
	///
	/// Instead of directly returning the [`Text`] line, this instead returns the [`Line`] type. You
	/// must then call [`.get(env)`](Line::get) on it to get the actual [`Text`]. This is because one
	/// of the replacements allows you to assign to [`Ast`](crate::Ast)s: Running an `Ast` requires
	/// a mutable reference to an [`Environment`], but `&mut self` already has a mutable reference,
	/// so `env.prompt().read_line(env)` doesn't actually work.
	///
	/// # Errors
	/// Any errors that occur when reading from stdin are bubbled upwards.
	pub fn read_line(&mut self) -> Result<Line<I, E>>
	where
		E: Encoding,
	{
		#[cfg(feature = "extensions")]
		match self.replacement.as_mut() {
			Some(PromptReplacement::Eof) => return Ok(Line(None)),
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
		Ok(Line(Some(ReadLineResultInner::Text(Text::new(line, self.flags)?))))
	}
}

/// Replacement functions.
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
impl<I: IntType, E: crate::value::text::Encoding> Prompt<'_, I, E> {
	/// Clears the currently set replacement, if any.
	pub fn reset_replacement(&mut self) {
		self.replacement = None;
	}

	/// Mimics stdin reaching EOF.
	///
	/// This clears any previous replacement.
	pub fn eof(&mut self) {
		self.replacement = Some(PromptReplacement::Eof);
	}

	/// Calling `PROMPT` will actually run `ast` and convert its return value to a [`Text`].
	///
	/// This clears any previous replacement.
	pub fn set_ast(&mut self, ast: Ast<I, E>) {
		self.replacement = Some(PromptReplacement::Computed(ast));
	}

	/// Adds `new_lines` to a queue of lines to be returned when `PROMPT` is called. If
	///
	/// This will clear any previous [`eof()`](Self::eof) and [`set_ast()`](Self::set_ast)
	/// replacements. However, it will _not_ clear previous `add_lines` replacements, and instead
	/// will simply add `new_lines` to the end.
	pub fn add_lines(&mut self, new_lines: &TextSlice<E>) {
		let lines = match self.replacement {
			Some(PromptReplacement::Buffered(ref mut lines)) => lines,
			_ => {
				// TODO: is there some way to make this cleaner?
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

			// SAFETY: `new_lines` is already a valid `TextSlice` of `E`, so any derivatives of it
			// are also valid `E`s. Additionally, `new_lines` already has its size checked, so anything
			// smaller than it is also within bounds.
			lines.push_back(unsafe { Text::new_unchecked(line) });
		}
	}
}
