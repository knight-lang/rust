use super::Environment;
use crate::value::Text;
use crate::Result;

#[cfg(feature = "extensions")]
use {
	crate::value::{Runnable, ToText, Value},
	crate::Ast,
	std::collections::VecDeque,
};

use std::io::{self, BufRead};

pub struct Prompt<'e> {
	pub(crate) default: Box<dyn BufRead + 'e + Send + Sync>,

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

pub struct ReadLineResult<'e>(Option<ReadLineResultInner<'e>>);
enum ReadLineResultInner<'e> {
	Text(Text),

	#[cfg(feature = "extensions")]
	Ast(Ast<'e>),
}

impl<'e> ReadLineResult<'e> {
	#[inline]
	pub fn get(self, env: &mut Environment<'e>) -> Result<Option<Text>> {
		match self.0 {
			None => Ok(None),
			Some(ReadLineResultInner::Text(text)) => Ok(Some(text)),

			#[cfg(feature = "extensions")]
			Some(ReadLineResultInner::Ast(ast)) => match ast.run(env)? {
				Value::Null => Ok(None),
				other => other.to_text(env).map(Some),
			},
		}
	}
}
#[cfg(feature = "extensions")]
impl<'e> Prompt<'e> {
	pub fn close(&mut self) {
		self.replacement = Some(PromptReplacement::Closed);
	}

	// ie, set the thing that does the computation
	pub fn set_ast(&mut self, ast: Ast<'e>) {
		self.replacement = Some(PromptReplacement::Computed(ast));
	}

	pub fn reset_replacement(&mut self) {
		self.replacement = None;
	}

	pub fn add_lines(&mut self, new_lines: &crate::value::text::TextSlice) {
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

		for line in (&**new_lines).split('\n') {
			let mut line = line.to_string();
			strip_ending(&mut line);
			lines.push_back(line.try_into().unwrap());
		}
	}
}

impl<'e> Prompt<'e> {
	#[cfg_attr(not(feature = "extensions"), inline)]
	pub fn read_line(&mut self) -> Result<ReadLineResult<'e>> {
		#[cfg(feature = "extensions")]
		match self.replacement.as_mut() {
			Some(PromptReplacement::Closed) => return Ok(ReadLineResult(None)),
			Some(PromptReplacement::Buffered(queue)) => {
				return Ok(ReadLineResult(queue.pop_front().map(ReadLineResultInner::Text)))
			}
			Some(PromptReplacement::Computed(ast)) => {
				return Ok(ReadLineResult(Some(ReadLineResultInner::Ast(ast.clone()))))
			}
			None => {}
		}

		let mut line = String::new();

		// If we read an empty line, return null.
		if self.default.read_line(&mut line)? == 0 {
			return Ok(ReadLineResult(None));
		}

		strip_ending(&mut line);
		Ok(ReadLineResult(Some(ReadLineResultInner::Text(Text::try_from(line)?))))
	}
}
