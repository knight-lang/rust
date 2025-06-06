use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser, VariableName};
use crate::program::JumpWhen;
use crate::value::KnString;
#[cfg(feature = "extensions")]
use crate::vm::opcode::DynamicAssignment;
use crate::vm::Opcode;
use crate::Options;

use super::SourceLocation;

pub struct Function;

fn simple_opcode_for(func: char, opts: &Options) -> Option<Opcode> {
	match func {
		// arity 0
		'P' => Some(Opcode::Prompt),
		'R' => Some(Opcode::Random),

		// arity 1
		'C' => Some(Opcode::Call),
		'Q' => Some(Opcode::Quit),
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

		// Extensions
		#[cfg(feature = "extensions")]
		'E' if opts.extensions.functions.eval => Some(Opcode::Eval),
		#[cfg(feature = "extensions")]
		'V' if opts.extensions.functions.value => Some(Opcode::Value),
		#[cfg(feature = "extensions")]
		'`' if opts.extensions.functions.system => Some(Opcode::System),

		_ => None,
	}
}

fn parse_argument<'path>(
	parser: &mut Parser<'_, '_, 'path, '_>,
	start: &SourceLocation<'path>,
	fn_name: char,
	arg: usize,
) -> Result<(), ParseError<'path>> {
	match parser.parse_expression() {
		Err(err) if matches!(err.kind, ParseErrorKind::EmptySource) => {
			return Err(ParseErrorKind::MissingArgument(fn_name, arg).error(*start));
		}
		other => other,
	}
}

fn parse_assignment<'path>(
	start: SourceLocation<'path>,
	parser: &mut Parser<'_, '_, 'path, '_>,
) -> Result<(), ParseError<'path>> {
	parser.strip_whitespace_and_comments();

	// TODO: handle `()` around variable name.
	match super::VariableName::parse(parser) {
		Err(err) if matches!(err.kind, ParseErrorKind::EmptySource) => {
			return Err(ParseErrorKind::MissingArgument('=', 1).error(start));
		}
		Err(err) => return Err(err),
		Ok(Some((name, location))) => {
			// try for a block, if so give it a name.
			parser.strip_whitespace_and_comments();
			if parser.peek().map_or(false, |c| c == 'B') {
				parser.strip_keyword_function();
				parse_block(start, parser, Some(name.clone()))?;
			} else {
				parse_argument(parser, &start, '=', 2)?;
			}
			// ew, cloning is not a good answer.
			let opts = (*parser.opts()).clone();
			unsafe { parser.compiler().set_variable(name, &opts) }
				.map_err(|err| err.error(location))?;
		}
		Ok(None) => {
			#[cfg(feature = "extensions")]
			{
				parser.strip_whitespace_and_comments();
				match parser.peek() {
					Some('R') => {
						if parser.opts().extensions.builtin_fns.assign_to_random {
							parser.strip_keyword_function();
							parse_argument(parser, &start, '=', 2)?;
							unsafe {
								parser.compiler.opcode_with_offset(
									Opcode::AssignDynamic,
									DynamicAssignment::Random as _,
								);
							}
							return Ok(());
						}
						// no else so we fallthru to the end
					}
					Some('O') | Some('P') | Some('$') => todo!("assign to builtins"),
					_ if parser.opts().extensions.builtin_fns.assign_to_strings => {
						parse_argument(parser, &start, '=', 1)?;
						parse_argument(parser, &start, '=', 2)?;
						unsafe {
							parser.compiler().opcode_without_offset(Opcode::SetDynamicVar);
						}
						return Ok(());
					}
					_ => {}
				}
			}

			return Err(ParseErrorKind::CanOnlyAssignToVariables.error(start));
		}
	}

	Ok(())
}

fn parse_block<'src, 'path>(
	start: SourceLocation<'path>,
	parser: &mut Parser<'_, 'src, 'path, '_>,
	name: Option<VariableName<'src>>,
) -> Result<(), ParseError<'path>> {
	// TODO: improve blocks later on by not having to jump over their definitions always.
	let jump_after = parser.compiler().defer_jump(JumpWhen::Always);

	let jump_index = parser.compiler().jump_index();
	parse_argument(parser, &start, 'B', 1)?;
	unsafe {
		parser.compiler().opcode_without_offset(Opcode::Return);
		jump_after.jump_to_current(parser.compiler());
	}

	parser.compiler().push_constant(crate::value::Block::new(jump_index).into());

	#[cfg(feature = "debugger")]
	parser.compiler().record_block(start, jump_index, name);
	Ok(())
}

impl Function {
	pub fn parse<'path>(parser: &mut Parser<'_, '_, 'path, '_>) -> Result<bool, ParseError<'path>> {
		// this should be reowrked ot allow for registering arbitrary functions, as it doesn't
		// support `X`s

		let (fn_name, full_name) = if let Some(fn_name) = parser.advance_if(char::is_uppercase) {
			(fn_name, parser.strip_keyword_function().unwrap_or_default())
		} else if let Some(chr) = parser.advance_if(|c| "!%&*+,-/:;<=>?[]^|~`".contains(c)) {
			(chr, "")
		} else {
			return Ok(false);
		};

		let start = parser.location();

		// Handle opcodes without anything special
		if let Some(simple_opcode) = simple_opcode_for(fn_name, parser.opts()) {
			debug_assert!(!simple_opcode.takes_offset()); // no simple opcodes take offsets

			for arg in 0..simple_opcode.arity() {
				parse_argument(parser, &start, fn_name, arg + 1)?;
			}

			unsafe {
				// todo: rename to simple opcode?
				parser.compiler.opcode_without_offset(simple_opcode);
			}

			return Ok(true);
		}

		// This is a simple op, except its arity is 0 so it never pops.
		if fn_name == 'D' {
			parse_argument(parser, &start, fn_name, 1)?;
			unsafe {
				// todo: rename to simple opcode?
				parser.compiler.opcode_without_offset(Opcode::Dump);
			}
			return Ok(true);
		}

		// Non-simple ones
		match fn_name {
			';' => {
				parse_argument(parser, &start, fn_name, 1)?;
				unsafe {
					parser.compiler.opcode_without_offset(Opcode::Pop);
				}
				parse_argument(parser, &start, fn_name, 1)?;
				Ok(true)
			}

			// technically not needed, as it wont ever get here. same with the if
			#[cfg(feature = "check-parens")]
			':' if parser.opts().check_parens => {
				parse_argument(parser, &start, fn_name, 1)?;
				return Ok(true);
			}
			'=' => parse_assignment(start, parser).and(Ok(true)),
			'B' => parse_block(start, parser, None).and(Ok(true)),
			'&' | '|' => {
				parse_argument(parser, &start, fn_name, 1)?;
				unsafe {
					parser.compiler().opcode_without_offset(Opcode::Dup);
				}
				let end = parser.compiler().defer_jump(if fn_name == '&' {
					JumpWhen::False
				} else {
					JumpWhen::True
				});
				unsafe {
					// delete the value we dont want
					parser.compiler().opcode_without_offset(Opcode::Pop);
				}
				parse_argument(parser, &start, fn_name, 2)?;
				unsafe {
					end.jump_to_current(parser.compiler());
				}
				Ok(true)
			}
			'I' => {
				parse_argument(parser, &start, fn_name, 1)?;
				let to_false = parser.compiler().defer_jump(JumpWhen::False);
				parse_argument(parser, &start, fn_name, 2)?;
				let to_end = parser.compiler().defer_jump(JumpWhen::Always);
				unsafe {
					to_false.jump_to_current(&mut parser.compiler());
				}
				parse_argument(parser, &start, fn_name, 3)?;
				unsafe {
					to_end.jump_to_current(parser.compiler());
				}
				Ok(true)
			}
			'W' => {
				let while_start = parser.compiler().jump_index();

				parse_argument(parser, &start, fn_name, 1)?;
				let deferred = parser.compiler().defer_jump(JumpWhen::False);
				parser.loops.push((while_start, vec![deferred]));

				parse_argument(parser, &start, fn_name, 3)?;
				unsafe {
					parser.compiler().opcode_without_offset(Opcode::Pop);
					parser.compiler().jump_to(JumpWhen::Always, while_start);
				}

				// jump all `break`s to the end
				for deferred in parser.loops.pop().unwrap().1 {
					unsafe {
						deferred.jump_to_current(parser.compiler());
					}
				}
				parser.compiler().push_constant(crate::Value::NULL);

				Ok(true)
			}

			#[cfg(feature = "extensions")]
			'X' if parser.opts().extensions.syntax.string_interpolation
				&& parser.advance_if('"').is_some() =>
			{
				let mut acc = String::new();
				let mut should_add = false;
				loop {
					match parser
						.advance()
						.ok_or_else(|| ParseErrorKind::MissingEndingQuote('"').error(start))?
					{
						'"' => break,
						'{' => {
							let gc = parser.gc();
							let (compiler, opts) = parser._compiler_opts();
							KnString::new_unvalidated(std::mem::take(&mut acc), gc)
								.compile(compiler, opts)?;

							parser.parse_expression()?;

							match parser.parse_expression() {
								Err(err) if matches!(err.kind, ParseErrorKind::UnknownTokenStart('}')) => {
									assert!(parser.advance_if('}').is_some()); // TODO: can this assertion fail?
								}
								other => return Err(ParseErrorKind::UnmatchedClosingBrace.error(start)),
							}

							// SAFETY: We compiled the previous string and this one
							unsafe {
								parser.compiler.opcode_without_offset(Opcode::Add);
							}

							should_add = true;
						}
						'\\' => match parser
							.advance()
							.ok_or_else(|| ParseErrorKind::MissingEndingQuote('"').error(start))?
						{
							'n' => acc.push('\n'),
							't' => acc.push('\t'),
							'r' => acc.push('\r'),
							'e' => acc.push('\x1B'),
							'\\' => acc.push('\\'),
							'\"' => acc.push('\"'),
							'\'' => acc.push('\''),
							'x' => {
								// TODO: make this cleaner
								let (hi, lo) = parser
									.advance_if(|c: char| c.is_ascii_hexdigit())
									.and_then(|hi| {
										parser.advance_if(|c: char| c.is_ascii_hexdigit()).map(|lo| (hi, lo))
									})
									.ok_or_else(|| ParseErrorKind::MissingEndingQuote('"').error(start))?;
								let joined = ((hi.to_digit(16).unwrap() << 4) | (lo.to_digit(16).unwrap()));
								let c = crate::strings::Character::new(
									joined as u8 as char,
									&parser.opts().encoding,
								)
								.expect(
									"TODO: an exception for invalid character, maybe an encoding error?",
								);
								acc.push(joined as u8 as char);
							}

							other => {
								return Err(ParseErrorKind::UnknownEscapeSequence(other).error(start))
							}
						},
						other => acc.push(other),
					}
				}
				use crate::program::Compilable;

				let gc = parser.gc();
				let (compiler, opts) = parser._compiler_opts();
				KnString::new_unvalidated(acc, gc).compile(compiler, opts)?;

				if should_add {
					unsafe {
						parser.compiler.opcode_without_offset(Opcode::Add);
					}
				}

				Ok(true)
			}

			// TODO: extensions lol
			#[cfg(feature = "extensions")]
			'X' => match full_name {
				"BREAK" if parser.opts().extensions.syntax.control_flow => {
					let deferred = parser.compiler().defer_jump(JumpWhen::Always);
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
						parser.compiler().jump_to(JumpWhen::Always, starting);
					}
					Ok(true)
				}
				_ => Err(ParseErrorKind::UnknownExtensionFunction(full_name.to_string()).error(start)),
			},
			_ => Err(ParseErrorKind::UnknownTokenStart(fn_name).error(start)),
		}
	}
}
