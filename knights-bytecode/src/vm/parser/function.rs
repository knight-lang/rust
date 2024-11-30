use crate::strings::StringSlice;
use crate::value::KString;
use crate::vm::{Opcode, ParseError, ParseErrorKind, Parseable, Parser};

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

unsafe impl Parseable for Function {
	fn parse(parser: &mut Parser<'_, '_>) -> Result<bool, ParseError> {
		// this should be reowrked ot allow for registering arbitrary functions

		let fn_name = if let Some(name) = parser.advance_if(char::is_uppercase) {
			parser.strip_keyword_function();
			name
		} else if let Some(chr) = parser.advance() {
			chr
		} else {
			return Ok(false);
		};

		let start = parser.location();

		// Handle opcodes without anything special
		if let Some(simple_opcode) = simple_opcode_for(fn_name) {
			debug_assert!(!simple_opcode.takes_offset()); // no simple opcodes take offsets

			for arg in 0..simple_opcode.arity() {
				match parser.parse_expression() {
					Err(err) if matches!(err.kind, ParseErrorKind::EmptySource) => {
						return Err(start.error(ParseErrorKind::MissingArgument(fn_name, arg + 1)));
					}
					other => other?,
				}
			}

			unsafe {
				// todo: rename to simple opcode?
				parser.builder.opcode_without_offset(simple_opcode);
			}

			return Ok(true);
		}

		todo!()

		// let Some(name) = Self::parse_name(parser)? else {
		// 	return Ok(false);
		// };

		// // TODO: ew, cloning the opts is icky as heck.
		// let opts = (*parser.opts()).clone();
		// parser.builder().get_variable(StringSlice::new_unvalidated(name), &opts);
		// Ok(true)
	}
}
