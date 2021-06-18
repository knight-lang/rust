use crate::{Value, Number, Text, Variable, value, func, Ast, func::Function};

static mut STREAM: *const u8 = std::ptr::null();

fn is_whitespace(c: u8) -> bool {
	c.is_ascii_whitespace() || c == b':'
		|| c == b'(' || c == b')'
		|| c == b'[' || c == b']'
		|| c == b'{' || c == b'}'
}

fn is_word_func(c: u8) -> bool {
	c.is_ascii_uppercase() || c == b'_'
}

unsafe fn peek() -> u8 {
	*STREAM
}

unsafe fn advance() {
	STREAM = STREAM.offset(1);
}

unsafe fn advance_peek() -> u8 {
	advance();
	peek()
}

unsafe fn peek_advance() -> u8 {
	let x = peek();
	advance();
	x
}

unsafe fn parse_strip() {
	debug_assert!(is_whitespace(peek()) || peek() == b'#');

	let mut c;

	loop {
		c = peek();

		if c == b'#' {
			while {c = advance_peek(); c } ==b'\n' || c == b'\0' {
				/* do nothing */
			}
		}

		if !is_whitespace(c) {
			break;
		}

		while is_whitespace(advance_peek()) {
			/* do nothing */
		}
	}
}

unsafe fn parse_number() -> Number {
	let mut c = peek();

	debug_assert!(c.is_ascii_digit());

	let mut number = (c - b'0') as Number;

	while { c = advance_peek(); c }.is_ascii_digit() {
		number = number * 10 + (c - b'0') as Number;
	}

	number
}

unsafe fn parse_text() -> *mut Text {
	let quote = peek_advance();
	let mut c;

	let start = STREAM;

	debug_assert!(quote == b'\'' || quote == b'\"');

	while quote != { c = peek_advance(); c } {
		#[cfg(not(feature = "reckless"))]
		if c == b'\0' {
			panic!("unterminated quote encountered: '{}'", c as char);
		}
	}

	crate::text::new_borrowed(start, (STREAM as usize) - (start as usize) - 1)
}

unsafe fn parse_variable() -> *mut Variable {
	let start = STREAM;

	debug_assert!(peek().is_ascii_lowercase() || peek() == b'_');

	let mut c;

	loop {
		c = advance_peek();

		if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_') {
			break;
		}
	}

	crate::env::fetch(start, (STREAM as usize) - (start as usize))
}

unsafe fn parse_ast(func: *const Function) -> *mut Ast {
	let ast = crate::ast::alloc(func);

	for i in 0..(*(*ast).func).arity {
		(*ast).args[i] = parse_value();

		#[cfg(not(feature="reckless"))]
		assert_ne!((*ast).args[i], value::UNDEFINED,
			"unable to parse argument {} for function '{}'", i, (*(*ast).func).name as char
		);
	}

	ast
}

unsafe fn strip_keyword() {
	while is_word_func(advance_peek()) {
		/* do nothing */
	}
}


unsafe fn parse_value() -> Value {
	let function;

	macro_rules! symbol_func {
		($name:ident) => {{
			function = &func::$name as *const _;
			advance();
		}};
	}

	macro_rules! word_func {
		($name:ident) => {{
			function = &func::$name as *const _;
			strip_keyword();
		}};
	}

	debug_assert_ne!(STREAM, std::ptr::null_mut());

	loop {
		match peek() {
			b'\0' => return value::UNDEFINED,
			b'\t' | b'\n' | b'\r' | b' ' |
			b'(' | b')' | b'[' | b']' | b'{' | b'}' | b':' | b'#' => {
				parse_strip();
				continue;
			},

			b'0'..=b'9' => return value::new_number(parse_number()),
			b'a'..=b'z' | b'_' => return value::new_variable(parse_variable()),
			b'\'' | b'\"' => return value::new_text(parse_text()),
			b'T' => {
				while is_word_func(advance_peek()) {}
				return value::TRUE;
			},
			b'F' => {
				while is_word_func(advance_peek()) {}
				return value::FALSE;
			},
			b'N' => {
				while is_word_func(advance_peek()) {}
				return value::NULL;
			},
			b'!' => symbol_func!(NOT),
			b'+' => symbol_func!(ADD),
			b'-' => symbol_func!(SUB),
			b'*' => symbol_func!(MUL),
			b'/' => symbol_func!(DIV),
			b'%' => symbol_func!(MOD),
			b'^' => symbol_func!(POW),
			b'?' => symbol_func!(EQL),
			b'<' => symbol_func!(LTH),
			b'>' => symbol_func!(GTH),
			b'&' => symbol_func!(AND),
			b'|' => symbol_func!(OR),
			b';' => symbol_func!(THEN),
			b'=' => symbol_func!(ASSIGN),
			b'`' => symbol_func!(SYSTEM),

			b'P' => word_func!(PROMPT),
			b'R' => word_func!(RANDOM),
			b'B' => word_func!(BLOCK),
			b'C' => word_func!(CALL),
			b'D' => word_func!(DUMP),
			b'E' => word_func!(EVAL),
			b'G' => word_func!(GET),
			b'I' => word_func!(IF),
			b'L' => word_func!(LENGTH),
			b'O' => word_func!(OUTPUT),
			b'Q' => word_func!(QUIT),
			b'S' => word_func!(SUBSTITUTE),
			b'W' => word_func!(WHILE),
			other => panic!("invalid source code byte '{:?}'", other as char)
		}

		return value::new_ast(parse_ast(function));
	}

}

pub unsafe fn parse(stream: *const u8) -> Value {
	STREAM = stream;

	parse_value()
}
