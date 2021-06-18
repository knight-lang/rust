use crate::{Text, Ast, Variable};

pub type Value = u64;
pub type Number = i64;

pub const FALSE: Value = 0;
pub const NULL: Value = 8;
pub const TRUE: Value = 16;
pub const UNDEFINED: Value = 24;

const SHIFT: usize = 3;
const TAG_CONSTANT: u64 = 0;
const TAG_NUMBER: u64   = 1;
const TAG_VARIABLE: u64 = 2;
const TAG_TEXT: u64     = 3;
const TAG_AST: u64      = 4;

macro_rules! tag {
	($expr:expr) => (($expr as Value) & 0b111)
}

macro_rules! unmask {
	($expr:expr) => (($expr as Value) & !0b111)
}

#[inline]
pub fn new_number(number: Number) -> Value {
	assert_eq!(number, (((number << SHIFT) as Value) >> SHIFT) as Number);

	((number as Value) << SHIFT) | TAG_NUMBER
}

#[inline]
pub fn new_boolean(boolean: bool) -> Value {
	(boolean as u64) << 4
}

#[inline]
pub fn new_text(text: *mut Text) -> Value {
	debug_assert_eq!(tag!(text), 0);

	text as Value | TAG_TEXT
}

#[inline]
pub fn new_variable(var: *mut Variable) -> Value {
	debug_assert_eq!(tag!(var), 0);

	var as Value | TAG_VARIABLE
}


#[inline]
pub fn new_ast(ast: *mut Ast) -> Value {
	debug_assert_eq!(tag!(ast), 0);

	ast as Value | TAG_AST
}

#[inline]
pub fn is_number(value: Value) -> bool {
	tag!(value) == TAG_NUMBER
}

#[inline]
pub fn is_boolean(value: Value) -> bool {
	value == TRUE || value == FALSE
}

#[inline]
pub fn is_text(value: Value) -> bool {
	tag!(value) == TAG_TEXT
}

#[inline]
pub fn is_variable(value: Value) -> bool {
	tag!(value) == TAG_VARIABLE
}

#[inline]
pub fn is_ast(value: Value) -> bool {
	tag!(value) == TAG_AST
}

#[inline]
pub unsafe fn as_number(value: Value) -> Number {
	debug_assert!(is_number(value));

	(value as Number) >> SHIFT
}

#[inline]
pub unsafe fn as_boolean(value: Value) -> bool {
	debug_assert!(is_boolean(value));

	value != FALSE
}

#[inline]
pub unsafe fn as_text(value: Value) -> *mut Text {
	debug_assert!(is_text(value));

	unmask!(value) as *mut Text
}

#[inline]
pub unsafe fn as_variable(value: Value) -> *mut Variable {
	debug_assert!(is_variable(value));

	unmask!(value) as *mut Variable
}

#[inline]
pub unsafe fn as_ast(value: Value) -> *mut Ast {
	debug_assert!(is_ast(value));

	unmask!(value) as *mut Ast
}


unsafe fn string_to_number(text: *const Text) -> Number {
	let mut ret: Number = 0;
	let mut ptr = crate::text::ptr(text as *mut Text) as *const u8;

	while (*ptr).is_ascii_whitespace() {
		ptr = ptr.offset(1);
	}

	let is_neg = *ptr == b'-';

	if is_neg || *ptr == b'+' {
		ptr = ptr.offset(1);
	}

	loop {
		let cur = *ptr.offset(0) - b'0';
		ptr = ptr.offset(1);

		if cur <= 9 {
			ret = ret * 10 + (cur as Number);
		} else {
			break;
		}
	}

	if is_neg {
		-ret
	} else {
		ret
	}
}

pub unsafe fn to_number(value: Value) -> Number {
	debug_assert_ne!(value, UNDEFINED);

	match tag!(value) {
		TAG_NUMBER             => as_number(value),
		TAG_CONSTANT           => (value == TRUE) as Number,
		TAG_TEXT               => string_to_number(as_text(value)),
		TAG_AST | TAG_VARIABLE => {
			let ran = run(value);
			let num = to_number(ran);
			free(ran);
			num
		},
		_ => unreachable!()
	}
}

pub unsafe fn to_boolean(value: Value) -> bool {
	debug_assert_ne!(value, UNDEFINED);

	match tag!(value) {
		TAG_CONSTANT           => value == TRUE,
		TAG_NUMBER             => value != TAG_NUMBER,
		TAG_TEXT               => (*as_text(value)).len != 0,
		TAG_AST | TAG_VARIABLE => {
			let ran = run(value);
			let bool = to_boolean(ran);
			free(ran);
			bool
		},
		_ => unreachable!()
	}
}

unsafe fn number_to_text(mut num: Number) -> *mut Text {
	static mut BUF: [u8; 22] = [b'\0'; 22];
	static mut NUM_TEXT: Text = Text {
		refcount: 0,
		flags: crate::text::FL_STATIC,
		len: 0,
		data: crate::text::TextData { alloc: std::ptr::null_mut() }
	};

	debug_assert!(num != 0 && num != 1);

	let end = (&mut BUF as *mut _ as *mut u8).offset((BUF.len() as isize) - 1);
	let mut ptr = end.offset(-1);
	let is_neg = num < 0;

	if is_neg {
		num *= -1;
	}

	loop {
		*ptr = b'0' + (num % 10) as u8;
		ptr = ptr.offset(-1);
		num /= 10;

		if num == 0 {
			break;
		}
	}

	if is_neg {
		*ptr = b'-';
		ptr = ptr.offset(-1);
	}

	NUM_TEXT.len = (ptr as usize) - (end as usize);
	NUM_TEXT.data.alloc = ptr;

	&mut NUM_TEXT as *mut Text
}

pub unsafe fn to_text(value: Value) -> *mut Text {
	macro_rules! unused {
		() => (crate::new_embed!(0,[0;23]));
	}
	static mut BUILTIN_STRINGS: [Text; TRUE as usize + 2] = [
		// 0: FALSE
		crate::new_embed!(5, [b'f', b'a', b'l', b's', b'e', b'\0',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
		// 1: 0
		crate::new_embed!(1, [b'0', b'\0',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
		// 2-7, unused
		unused!(), unused!(), unused!(), unused!(), unused!(), unused!(),
		// 8: NULL
		crate::new_embed!(4, [b'n', b'u', b'l', b'l', b'\0',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
		// 9: 1
		crate::new_embed!(1, [b'1', b'\0',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
		// 10-15, unused
		unused!(), unused!(), unused!(), unused!(), unused!(), unused!(),
		// 16: TRUE
		crate::new_embed!(4, [b't', b'r', b'u', b'e', b'\0',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
		// 17: 2
		crate::new_embed!(1, [b'2', b'\0',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
	];

	debug_assert_ne!(value, UNDEFINED);

	if value <= new_number(2) {
		return &mut BUILTIN_STRINGS[value as usize] as *mut Text;
	}

	match tag!(value) {
		TAG_NUMBER => number_to_text(as_number(value)),
		TAG_TEXT => crate::text::clone(as_text(value)),
		TAG_AST | TAG_VARIABLE => {
			let ran = run(value);
			let text = to_text(ran);
			free(ran);
			text
		},
		TAG_CONSTANT | _ => unreachable!()
	}
}

pub unsafe fn dump(value: Value) {
	match tag!(value) {
		TAG_CONSTANT =>
			match value {
				TRUE => print!("Boolean(true)"),
				FALSE => print!("Boolean(false)"),
				NULL => print!("Null()"),
				UNDEFINED => print!("(Undefined)"),
				_ => unreachable!()
			},

		TAG_NUMBER => print!("Number({})", as_number(value)),
		TAG_TEXT   => {
			let text = as_text(value);

			print!("String({})", std::str::from_utf8_unchecked(std::slice::from_raw_parts(crate::text::ptr(text), (*text).len as usize)))
		},
		TAG_VARIABLE => print!("Variable({})", (*as_variable(value)).name_str()),
		TAG_AST => {
			let ast = as_ast(value);
			print!("Function({}", (*(*ast).func).name);

			for i in 0..(*(*ast).func).arity {
				print!(", ");
				dump((*ast).args[i]);
			}

			print!(")");
		},
		_ => unreachable!()
	}
}

pub unsafe fn run(value: Value) -> Value {
	debug_assert_ne!(value, UNDEFINED);

	match tag!(value) {
		TAG_AST => crate::ast::run(as_ast(value)),
		TAG_VARIABLE => crate::env::run(as_variable(value)),
		TAG_TEXT | TAG_NUMBER | TAG_CONSTANT => {
			if is_text(value) {
				let _ = as_text(value).clone();
			}

			value
		},
		_ => unreachable!()
	}
}

pub unsafe fn clone(value: Value) -> Value {
	debug_assert_ne!(value, UNDEFINED);

	match tag!(value) {
		TAG_AST => (*as_ast(value)).refcount += 1,
		TAG_TEXT => (*as_text(value)).refcount += 1,
		TAG_CONSTANT | TAG_NUMBER | TAG_VARIABLE => {}
		_ => unreachable!()
	}

	value
}

pub unsafe fn free(value: Value) {
	debug_assert_ne!(value, UNDEFINED);

	match tag!(value) {
		TAG_CONSTANT | TAG_NUMBER | TAG_VARIABLE => {},
		TAG_TEXT => crate::text::free(as_text(value)),
		TAG_AST => crate::ast::free(as_ast(value)),
		_ => unreachable!()
	}
}
