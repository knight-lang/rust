use crate::strings::StringSlice;
use crate::value::KString;
use crate::vm::program::{DeferredJump, JumpWhen};
use crate::vm::{Opcode, ParseError, ParseErrorKind, Parseable, Parser};

use super::SourceLocation;

pub struct Function;

fn simple_opcode_for(func: char) -> Option<Opcode> {
	match func {
		// arity 0
		'P' => Some(Opcode::Prompt),
		'R' => Some(Opcode::Random),

		// arity 1
		'C' => Some(Opcode::Call),
		'Q' => Some(Opcode::Quit),
		'D' => Some(Opcode::Dump),
		'O' => Some(Opcode::Output),
		'L' => Some(Opcode::Length),
		'!' => Some(Opcode::Not),
		'~' => Some(Opcode::Negate),
		'A' => Some(Opcode::Ascii),
		',' => Some(Opcode::Box),
		'[' => Some(Opcode::Head),
		']' => Some(Opcode::Tail),

		// arity 2
		'+' => Some(Opcode::Add),
		'-' => Some(Opcode::Sub),
		'*' => Some(Opcode::Mul),
		'/' => Some(Opcode::Div),
		'%' => Some(Opcode::Mod),
		'^' => Some(Opcode::Pow),
		'<' => Some(Opcode::Lth),
		'>' => Some(Opcode::Gth),
		'?' => Some(Opcode::Eql),

		// arity 3
		'G' => Some(Opcode::Get),

		// arity 4
		'S' => Some(Opcode::Set),

		_ => None,
	}
}

fn parse_argument(
	parser: &mut Parser<'_, '_>,
	start: &SourceLocation,
	fn_name: char,
	arg: usize,
) -> Result<(), ParseError> {
	match parser.parse_expression() {
		Err(err) if matches!(err.kind, ParseErrorKind::EmptySource) => {
			return Err(start.clone().error(ParseErrorKind::MissingArgument(fn_name, arg)));
		}
		other => other,
	}
}

unsafe impl Parseable for Function {
	fn parse(parser: &mut Parser<'_, '_>) -> Result<bool, ParseError> {
		// this should be reowrked ot allow for registering arbitrary functions, as it doesn't
		// support `X`s

		let (fn_name, full_name) = if let Some(fn_name) = parser.advance_if(char::is_uppercase) {
			(fn_name, parser.strip_keyword_function().unwrap_or_default())
		} else if let Some(chr) = parser.advance() {
			(chr, "")
		} else {
			return Ok(false);
		};

		let start = parser.location();

		// Handle opcodes without anything special
		if let Some(simple_opcode) = simple_opcode_for(fn_name) {
			debug_assert!(!simple_opcode.takes_offset()); // no simple opcodes take offsets

			for arg in 0..simple_opcode.arity() {
				parse_argument(parser, &start, fn_name, arg + 1)?;
			}

			unsafe {
				// todo: rename to simple opcode?
				parser.builder.opcode_without_offset(simple_opcode);
			}

			return Ok(true);
		}

		// Non-simple ones
		match fn_name {
			';' => {
				parse_argument(parser, &start, fn_name, 1)?;
				unsafe {
					parser.builder.opcode_without_offset(Opcode::Pop);
				}
				parse_argument(parser, &start, fn_name, 1)?;
				Ok(true)
			}
			'=' => {
				parser.strip_whitespace_and_comments();

				match super::variable::Variable::parse_name(parser) {
					Err(err) if matches!(err.kind, ParseErrorKind::EmptySource) => {
						Err(start.error(ParseErrorKind::MissingArgument('=', 1)))
					}
					Err(err) => Err(err),
					Ok(Some(name)) => {
						parse_argument(parser, &start, fn_name, 2)?;
						// ew, cloning is not a good answer.
						let opts = (*parser.opts()).clone();
						// i dont like this new_unvalidated. TODO: fix it.
						let name = StringSlice::new_unvalidated(name);
						unsafe { parser.builder().set_variable(name, &opts) }
							.expect("<todo, the name should already have been checked. remove this.>");
						Ok(true)
					}
					Ok(None) => {
						#[cfg(feature = "extensions")]
						{
							todo!("Assign to OUTPUT, PROMPT, RANDOM, strings, $, and more.");
						}

						Err(start.error(ParseErrorKind::CanOnlyAssignToVariables))
					}
				}
			}
			'B' => todo!("blocks"),
			'&' | '|' => {
				parse_argument(parser, &start, fn_name, 1)?;
				unsafe {
					parser.builder().opcode_without_offset(Opcode::Dup);
				}
				let end = parser.builder().defer_jump(if fn_name == '&' {
					JumpWhen::False
				} else {
					JumpWhen::True
				});
				parse_argument(parser, &start, fn_name, 2)?;
				unsafe {
					end.jump_to_current(parser.builder());
				}
				Ok(true)
			}
			'I' => {
				parse_argument(parser, &start, fn_name, 1)?;
				let to_false = parser.builder().defer_jump(JumpWhen::False);
				parse_argument(parser, &start, fn_name, 2)?;
				let to_end = parser.builder().defer_jump(JumpWhen::False);
				unsafe {
					to_false.jump_to_current(&mut parser.builder());
				}
				parse_argument(parser, &start, fn_name, 3)?;
				unsafe {
					to_end.jump_to_current(parser.builder());
				}
				Ok(true)
			}
			'W' => {
				let while_start = parser.builder().jump_index();

				parse_argument(parser, &start, fn_name, 1)?;
				let deferred = parser.builder().defer_jump(JumpWhen::False);
				parser.loops.push((while_start, vec![deferred]));

				parse_argument(parser, &start, fn_name, 3)?;
				unsafe {
					parser.builder().jump_to(JumpWhen::Always, while_start);
				}

				// jump all `break`s to the end
				for deferred in parser.loops.pop().unwrap().1 {
					unsafe {
						deferred.jump_to_current(parser.builder());
					}
				}

				Ok(true)
			}
			// TODO: extensions lol
			#[cfg(feature = "extensions")]
			'X' => match full_name {
				"BREAK" if parser.opts().extensions.syntax.control_flow => {
					let deferred = parser.builder().defer_jump(JumpWhen::Always);
					parser
						.loops
						.last_mut()
						.expect("<todo: exception when `break` when nothing to break, or in a funciton?>")
						.1
						.push(deferred);
					Ok(true)
				}
				"CONTINUE" if parser.opts().extensions.syntax.control_flow => {
					let starting = parser
						.loops
						.last()
						.expect("<todo: exception when `break` when nothing to break, or in a funciton?>")
						.0;
					unsafe {
						parser.builder().jump_to(JumpWhen::Always, starting);
					}
					Ok(true)
				}
				_ => Err(start.error(ParseErrorKind::UnknownExtensionFunction(full_name.to_string()))),
			},

			_ => todo!(),
		}
	}
}
