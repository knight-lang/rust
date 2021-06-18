use crate::{Value, value, text, Number, Text};
use crate::shared::{strcmp, memcpy};

pub struct Function {
	pub arity: usize,
	pub name: u8,
	pub func: unsafe fn(*const Value) -> Value
}

pub const MAX_ARGC: usize = 4;

pub unsafe fn startup() {
	extern "C" {
		fn srand(seed: u32);
		fn time(_: *const u8) -> *mut u8;
	}

	srand(time(std::ptr::null()) as u32);
}

macro_rules! declare_function {
	($fn:ident, $struct:ident, $arity:literal, $name:literal, $args:pat, $body:block) => {
		pub static $struct: Function = Function { arity: $arity, name: $name, func: $fn };
		pub unsafe fn $fn($args: *const Value) -> Value $body
	}
}

declare_function!(prompt, PROMPT, 0, b'P', _, {
	use std::io::stdin;

	let mut text = String::new();
	stdin().read_line(&mut text).unwrap();

	if text.bytes().last() == Some(b'\n') {
		text.pop();
		if text.bytes().last() == Some(b'\r') {
			text.pop();
		}
	}

	let len = text.len();
	let ptr = text.as_mut_ptr();
	std::mem::forget(text);

	value::new_text(
		if len == 0 {
			&mut text::EMPTY as _
		} else {
			text::new_owned(ptr, len)
		}
	)
});

declare_function!(random, RANDOM, 0, b'R', _, {
	extern "C" {
		fn rand() -> i32;
	}

	value::new_number(rand() as Number)
});

declare_function!(eval, EVAL, 1, b'E', args, {
	let text = value::to_text(*args);
	let ret = crate::run(text::ptr(text));

	text::free(text);

	ret
});

declare_function!(block, BLOCK, 1, b'B', args, {
	value::clone(*args)
});

declare_function!(call, CALL, 1, b'C', args, {
	let ran = value::run(*args);
	let result = value::run(ran);

	value::free(ran);

	result
});

declare_function!(system, SYSTEM, 1, b'`', args, {
	use std::alloc::{alloc, realloc, Layout};

	enum FILE {}

	extern "C" {
		fn fopen(str: *mut u8, mode: *const u8) -> *mut FILE;
		fn fread(ptr: *mut u8, len: usize, items: usize, stream: *mut FILE) -> usize;

		#[cfg(not(feature="reckless"))]
		fn ferror(file: *const FILE) -> i32;
	}

	let command = value::to_text(*args);
	let str = text::ptr(command);
	let stream = fopen(str, b"r\0" as _);

	#[cfg(not(feature="reckless"))]
	assert!(!stream.is_null(), "unable to execute command '{}'", (*command).str());

	text::free(command);

	let mut tmp;
	let mut cap = 2048;
	let mut len = 0;
	let mut result = alloc(Layout::from_size_align_unchecked(cap, 1));

	#[cfg(not(feature="reckless"))]
	assert!(!result.is_null(), "alloc failed");

	while 0 != { tmp = fread(result.offset(len as isize), 1, cap - len, stream); tmp } {
		len += tmp;

		if len == cap {
			cap *= 2;

			result = realloc(result, Layout::from_size_align_unchecked(cap / 2, 1), cap);
			#[cfg(not(feature="reckless"))]
			assert!(!result.is_null(), "realloc failed");
		}
	}

	#[cfg(not(feature="reckless"))]
	assert_eq!(ferror(stream), 0, "unable to read command stream");

	result = realloc(result, Layout::from_size_align_unchecked(cap, 1), len + 1);
	#[cfg(not(feature="reckless"))]
	assert!(!result.is_null(), "realloc failed");

	*result.offset(len as isize) = b'\0';

	value::new_text(text::new_owned(result, len))
});

declare_function!(quit, QUIT, 1, b'Q', args, {
	std::process::exit(value::to_number(*args) as i32)
});

declare_function!(not, NOT, 1, b'!', args, {
	value::new_boolean(!value::to_boolean(*args))
});

declare_function!(length, LENGTH, 1, b'L', args, {
	let text = value::to_text(*args);
	let len = (*text).len;

	text::free(text);

	value::new_number(len as _)
});

declare_function!(dump, DUMP, 1, b'D', args, {
	let val = value::run(*args);

	value::dump(val);
	println!();

	val
});

declare_function!(output, OUTPUT, 1, b'O', args, {
	let text = value::to_text(*args);
	let len = (*text).len;
	let ptr = text::ptr(text);

	if len == 0 {
		println!();
	} else if *ptr.offset(len as isize - 2) == b'\\' {
		print!("{}", &(*text).str()[..len - 2]);
	} else {
		println!("{}", (*text).str());
	}

	value::NULL
});

unsafe fn add_text(lhs: *mut Text, rhs: *mut Text) -> Value {
	let lhslen;
	let rhslen;

	if { lhslen = (*lhs).len; lhslen } == 0 {
		debug_assert_eq!(lhs, &mut text::EMPTY as _);
		return value::new_text(text::clone_static(rhs));
	}

	if { rhslen = (*rhs).len; rhslen } == 0 {
		debug_assert_eq!(rhs, &mut text::EMPTY as _);
		return value::new_text(lhs);
	}

	let mut hash = crate::shared::hash(text::ptr(lhs), (*lhs).len);
	hash = crate::shared::hash_acc(text::ptr(rhs), (*rhs).len, hash);

	let len = lhslen + rhslen;
	let mut text = text::text_cache_lookup(hash, len);

'free_and_return: loop {
'allocate_and_cache: loop {
	if text.is_null() {
		break 'allocate_and_cache;
	}

	let mut cached = text::ptr(text);
	let mut tmp = text::ptr(lhs);

	for i in 0..(*lhs).len {
		if *cached != *tmp.offset(i as isize) {
			break 'allocate_and_cache;
		}
		cached = cached.offset(1);
	}

	tmp = text::ptr(rhs);

	for i in 0..(*rhs).len {
		if *cached != *tmp.offset(i as isize) {
			break 'allocate_and_cache;
		}
		cached = cached.offset(1);
	}

	text = text::clone(text);
	break 'free_and_return;
} // allocate_and_cache
	text = text::alloc(len);
	let str = text::ptr(text);

	*str.offset(len as isize) = b'\0';
	memcpy(str, text::ptr(lhs), lhslen);
	memcpy(str.offset(lhslen as isize), text::ptr(rhs), rhslen);

	break;
} // free_and_return
	
	text::free(lhs);
	text::free(rhs);

	value::new_text(text)
}

declare_function!(add, ADD, 2, b'+', args, {
	let lhs = *args;

	if value::is_text(lhs) {
		return add_text(value::as_text(lhs), value::to_text(*args.offset(1)));
	}

	#[cfg(not(feature="reckless"))]
	assert!(value::is_number(lhs), "can only add to strings and numbers");

	let augend = value::as_number(lhs);
	let addend = value::to_number(*args.offset(1));

	value::new_number(augend + addend)
});

declare_function!(sub, SUB, 2, b'-', args, {
	let lhs = value::run(*args);

	#[cfg(not(feature="reckless"))]
	assert!(value::is_number(lhs), "can only subtract from numbers!");

	let minuend = value::as_number(lhs);
	let subtrahend = value::to_number(*args.offset(1));

	value::new_number(minuend - subtrahend)
});

unsafe fn mul_text(lhs: *mut Text, times: usize) -> Value {
	let lhslen = (*lhs).len;

	if lhslen == 0 || times == 0 {
		let empty = &mut text::EMPTY as *mut Text;

		if lhslen != 0 {
			text::free(lhs);
		} else {
			debug_assert_eq!(lhs, empty);
		}

		return value::new_text(empty);
	}

	if times == 1{
		return value::new_text(lhs);
	}

	let len = lhslen * times;

	let text = text::alloc(len);
	let mut str = text::ptr(text);
	*str.offset(len as isize) = b'\0';

	for _ in 0..times {
		memcpy(str, text::ptr(lhs), lhslen);
		str = str.offset(lhslen as isize);
	}

	value::new_text(text)
}

declare_function!(mul, MUL, 2, b'*', args, {
	let lhs = *args;

	if value::is_text(lhs) {
		return mul_text(value::as_text(lhs), value::to_number(*args.offset(1)) as usize);
	}

	#[cfg(not(feature="reckless"))]
	assert!(value::is_number(lhs), "can only add to strings and numbers");

	let multiplicand = value::as_number(lhs);
	let multiplier = value::to_number(*args.offset(1));

	value::new_number(multiplicand + multiplier)
});

declare_function!(div, DIV, 2, b'/', args, {
	let lhs = value::run(*args);

	#[cfg(not(feature="reckless"))]
	assert!(value::is_number(lhs), "can only divide numbers");

	let dividend = value::as_number(lhs);
	let divisor = value::to_number(*args.offset(1));

	#[cfg(not(feature="reckless"))]
	assert_ne!(divisor, 0, "attempted to divide by zero");

	value::new_number(dividend / divisor)
});

declare_function!(r#mod, MOD, 2, b'%', args, {
	let lhs = value::run(*args);

	#[cfg(not(feature="reckless"))]
	assert!(value::is_number(lhs), "can only modulo numbers");

	let number = value::as_number(lhs);
	let base = value::to_number(*args.offset(1));

	#[cfg(not(feature="reckless"))]
	assert_ne!(base, 0, "attempted to modulo by zero");

	value::new_number(number % base)
});

declare_function!(pow, POW, 2, b'^', args, {
	let lhs = value::run(*args);

	#[cfg(not(feature="reckless"))]
	assert!(value::is_number(lhs), "can only exponentiate numbers");

	let base = value::as_number(lhs);
	let exponent = value::to_number(*args.offset(1));

	#[cfg(not(feature="reckless"))]
	assert!(base != 0 || exponent > 0, "attempted to exponentiate zero by a negative value.");

	value::new_number((base as f64).powf(exponent as f64) as Number)
});

declare_function!(lth, LTH, 2, b'<', args, {
	let lhs = value::run(*args);
	let less;

	if value::is_text(lhs) {
		let lstr = value::as_text(lhs);
		let rstr = value::to_text(*args.offset(1));

		less = strcmp(text::ptr(lstr), text::ptr(rstr)) < 0;

		text::free(lstr);
		text::free(rstr);
	} else if value::is_number(lhs) {
		less = value::as_number(lhs) < value::to_number(*args.offset(1));
	} else {
		#[cfg(not(feature="reckless"))]
		assert!(value::is_boolean(lhs), "can only compare to numbers, strings, and booleans");

		// note that `== FALSE` needs to be after, otherwise rhs wont be run.
		less = value::to_boolean(*args.offset(1)) && lhs == value::FALSE;
	}

	value::new_boolean(less)
});

declare_function!(gth, GTH, 2, b'>', args, {
	let lhs = value::run(*args);
	let greater;

	if value::is_text(lhs) {
		let lstr = value::as_text(lhs);
		let rstr = value::to_text(*args.offset(1));

		greater = strcmp(text::ptr(lstr), text::ptr(rstr)) > 0;

		text::free(lstr);
		text::free(rstr);
	} else if value::is_number(lhs) {
		greater = value::as_number(lhs) > value::to_number(*args.offset(1));
	} else {
		#[cfg(not(feature="reckless"))]
		assert!(value::is_boolean(lhs), "can only compare to numbers, strings, and booleans");

		// note that `== FALSE` needs to be after, otherwise rhs wont be run.
		greater = !value::to_boolean(*args.offset(1)) && lhs == value::TRUE;
	}

	value::new_boolean(greater)
});

declare_function!(eql, EQL, 2, b'?', args, {
	let lhs = value::run(*args);
	let rhs = value::run(*args.offset(1));
	let mut eql;

	debug_assert_ne!(lhs, value::UNDEFINED);
	debug_assert_ne!(rhs, value::UNDEFINED);

	if { eql = lhs == rhs; eql } {
		/* do nothing */
	} else if !{ eql = value::is_text(lhs) && value::is_text(rhs); eql } {
		/* do nothing */
	} else {
		eql = text::is_equal(value::as_text(lhs), value::as_text(rhs));
	}

	value::free(lhs);
	value::free(rhs);

	value::new_boolean(eql)
});

declare_function!(and, AND, 2, b'&', args, {
	let lhs = value::run(*args);

	// return the lhs if its falsey.
	if !value::to_boolean(lhs) {
		return lhs;
	}

	value::free(lhs);
	value::run(*args.offset(1))
});

declare_function!(or, OR, 2, b'|', args, {
	let lhs = value::run(*args);

	// return the lhs if its truthy.
	if value::to_boolean(lhs) {
		return lhs;
	}

	value::free(lhs);
	value::run(*args.offset(1))
});

declare_function!(then, THEN, 2, b';', args, {
	value::free(value::run(*args));

	value::run(*args.offset(1))
});

declare_function!(assign, ASSIGN, 2, b'=', args, {
	#[cfg(not(feature="reckless"))]
	assert!(value::is_variable(*args));

	let value = value::run(*args.offset(1));
	crate::env::assign(value::as_variable(*args), value::clone(value));

	value
});

declare_function!(r#while, WHILE, 2, b'W', args, {
	while value::to_boolean(*args) {
		value::free(value::run(*args.offset(1)));
	}

	value::NULL
});

declare_function!(r#if, IF, 3, b'I', args, {
	let idx = value::to_boolean(*args);

	value::run(*args.offset(1 + !idx as isize))
});

declare_function!(get, GET, 3, b'G', args, {
	let text = value::to_text(*args);
	let start = value::to_number(*args.offset(1)) as usize;
	let len = value::to_number(*args.offset(2)) as usize;

	let substr;

	// if we're getting past the end of the array, simply return the
	// empty text.
	if (*text).len <= start {
		substr = &mut text::EMPTY as *mut Text;
	} else {
		// if the total len is too much, simply wrap around to the end.
		#[cfg(not(feature="reckless"))]
		assert!((*text).len < start + len, "ending position is too large!");

		substr = text::new_borrowed(text::ptr(text).offset(start as isize), len);
	}

	text::free(text);

	value::new_text(substr)
});

declare_function!(substitute, SUBSTITUTE, 4, b'S', args, {
	let text = value::to_text(*args);
	let start = value::to_number(*args.offset(1)) as usize;
	let mut amnt = value::to_number(*args.offset(2)) as usize;
	let repl = value::to_text(*args.offset(3));

	let text_len = (*text).len;
	let repl_len = (*repl).len;

	// if it's out of bounds, die.
	#[cfg(not(feature="reckless"))]
	assert!(text_len < start, "index '{}' out of bounds (length={})", start, text_len);

	if text_len <= start + amnt {
		amnt = text_len - start;
	}

	if start == 0 && repl_len == 0 {
		let result = text::new_borrowed(text::ptr(text).offset(amnt as isize), text_len - amnt);
		text::free(text);
		return value::new_text(result);
	}

	// TODO: you could also check for caching here first.
	let result = text::alloc(text_len - amnt + repl_len);
	let mut ptr = text::ptr(result);

	memcpy(ptr, text::ptr(text), start);
	ptr = ptr.offset(start as isize);

	memcpy(ptr, text::ptr(repl), repl_len);
	ptr = ptr.offset(repl_len as isize);
	text::free(repl);

	memcpy(ptr, text::ptr(text).offset((start + amnt) as _), text_len - amnt - start + 1); // `+1` so we copy the `\0` too.

	text::free(text);
	text::cache(result);

	return value::new_text(result);
});

